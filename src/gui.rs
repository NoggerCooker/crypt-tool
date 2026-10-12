use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use age::{Encryptor, Decryptor, scrypt::Identity, secrecy::SecretString};
use anyhow::{Result, Context};
use eframe::egui;
use rfd::FileDialog;
use zeroize::Zeroizing;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 380.0])
            .with_min_inner_size([480.0, 320.0])
            .with_title("crypt-tool"),
        ..Default::default()
    };
    eframe::run_native(
        "crypt-tool",
        options,
        Box::new(|_cc| Ok(Box::new(CryptApp::default()))),
    ).map_err(|e| anyhow::anyhow!("{}", e))
}

#[derive(Default)]
struct CryptApp {
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    passphrase: String,
    passphrase_confirm: String,
    show_passphrase: bool,
    mode: Mode,
    status: Status,
    rx: Option<mpsc::Receiver<Status>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Encrypt,
    Decrypt,
}

impl Default for Mode {
    fn default() -> Self { Mode::Encrypt }
}

#[derive(Debug, Clone)]
enum Status {
    Idle,
    Working(String),
    Success(String),
    Error(String),
}

impl Default for Status {
    fn default() -> Self { Status::Idle }
}

impl CryptApp {
    fn pick_input(&mut self) {
        let file = FileDialog::new()
            .set_title(match self.mode {
                Mode::Encrypt => "Datei zum Verschlüsseln wählen",
                Mode::Decrypt => "Datei zum Entschlüsseln wählen (.age)",
            })
            .pick_file();
        if let Some(path) = file {
            self.input_path = Some(path.clone());
            // Auto-set output path
            self.output_path = match self.mode {
                Mode::Encrypt => Some(path.with_extension("age")),
                Mode::Decrypt => {
                    let mut p = path.clone();
                    p.set_extension("");
                    Some(p)
                }
            };
            self.status = Status::Idle;
        }
    }

    fn pick_output(&mut self) {
        let file = FileDialog::new()
            .set_title("Speicherort wählen")
            .set_file_name(self.output_path.as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(""))
            .save_file();
        if let Some(path) = file {
            self.output_path = Some(path);
        }
    }

    fn start_operation(&mut self) {
        if self.passphrase.is_empty() {
            self.status = Status::Error("Passphrase eingeben".into());
            return;
        }
        if self.passphrase != self.passphrase_confirm {
            self.status = Status::Error("Passphrasen stimmen nicht überein".into());
            return;
        }
        let Some(input) = self.input_path.clone() else {
            self.status = Status::Error("Eingabedatei wählen".into());
            return;
        };
        let Some(output) = self.output_path.clone() else {
            self.status = Status::Error("Ausgabedatei wählen".into());
            return;
        };
        if input == output {
            self.status = Status::Error("Ein- und Ausgabedatei dürfen nicht identisch sein".into());
            return;
        }

        let mode = self.mode;
        // Convert to Zeroizing for crypto operations (zeroed on drop)
        let passphrase = Zeroizing::new(self.passphrase.clone());

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.status = Status::Working("Arbeite...".into());

        thread::spawn(move || {
            let result = match mode {
                Mode::Encrypt => encrypt_file(&input, &output, &passphrase),
                Mode::Decrypt => decrypt_file(&input, &output, &passphrase),
            };
            let _ = tx.send(match result {
                Ok(_) => Status::Success(format!("Fertig: {}", output.display())),
                Err(e) => Status::Error(e.to_string()),
            });
        });
    }

    fn poll_result(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(status) = rx.try_recv() {
                self.status = status;
                self.rx = None;
            }
        }
    }
}

impl eframe::App for CryptApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_result();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("crypt-tool");
            ui.separator();

            // Mode tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.mode, Mode::Encrypt, "🔒 Verschlüsseln");
                ui.selectable_value(&mut self.mode, Mode::Decrypt, "🔓 Entschlüsseln");
            });
            ui.add_space(8.0);

            // Input file
            ui.horizontal(|ui| {
                ui.label("Eingabe:");
                let path_str = self.input_path.as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "—".into());
                ui.text_edit_singleline(&mut path_str.clone());
                if ui.button("📁 Wählen").clicked() {
                    self.pick_input();
                }
            });

            // Output file
            ui.horizontal(|ui| {
                ui.label("Ausgabe:");
                let mut path_str = self.output_path.as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "—".into());
                let response = ui.text_edit_singleline(&mut path_str);
                if response.changed() {
                    self.output_path = Some(PathBuf::from(&path_str));
                }
                if ui.button("📁 Wählen").clicked() {
                    self.pick_output();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Passphrase
            ui.horizontal(|ui| {
                ui.label("Passphrase:");
                if self.show_passphrase {
                    ui.text_edit_singleline(&mut self.passphrase);
                } else {
                    ui.add(egui::TextEdit::singleline(&mut self.passphrase).password(true));
                }
                if ui.button(if self.show_passphrase { "🙈" } else { "👁" }).clicked() {
                    self.show_passphrase = !self.show_passphrase;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Wiederholen:");
                if self.show_passphrase {
                    ui.text_edit_singleline(&mut self.passphrase_confirm);
                } else {
                    ui.add(egui::TextEdit::singleline(&mut self.passphrase_confirm).password(true));
                }
            });

            ui.add_space(12.0);

            // Action button
            let btn_text = match self.mode {
                Mode::Encrypt => "🔒 Verschlüsseln",
                Mode::Decrypt => "🔓 Entschlüsseln",
            };
            let btn = egui::Button::new(egui::RichText::new(btn_text).size(16.0)).fill(egui::Color32::from_rgb(60, 130, 60));
            if ui.add_sized([ui.available_width(), 44.0], btn).clicked() {
                self.start_operation();
            }

            ui.add_space(8.0);

            // Status
            match &self.status {
                Status::Idle => {}
                Status::Working(msg) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(msg);
                    });
                }
                Status::Success(msg) => {
                    ui.colored_label(egui::Color32::GREEN, format!("✓ {}", msg));
                }
                Status::Error(msg) => {
                    ui.colored_label(egui::Color32::RED, format!("✗ {}", msg));
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.small("Basiert auf age (scrypt + ChaCha20-Poly1305) — Streaming, keine Temp-Dateien");
        });

        // Request repaint while working
        if matches!(self.status, Status::Working(_)) {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

/// Core crypto functions (shared with CLI)
fn encrypt_file(input: &Path, output: &Path, passphrase: &Zeroizing<String>) -> Result<()> {
    let mut in_file = BufReader::new(File::open(input).context("open input")?);
    let out_file = BufWriter::new(File::create(output).context("create output")?);

    let passphrase = SecretString::new(passphrase.to_string().into_boxed_str());
    let encryptor = Encryptor::with_user_passphrase(passphrase);
    let mut writer = encryptor.wrap_output(out_file)?;

    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = in_file.read(&mut buf).context("read input")?;
        if n == 0 { break; }
        writer.write_all(&buf[..n]).context("write encrypted")?;
    }
    writer.finish().context("finalize encryption")?;
    Ok(())
}

fn decrypt_file(input: &Path, output: &Path, passphrase: &Zeroizing<String>) -> Result<()> {
    let mut in_file = BufReader::new(File::open(input).context("open input")?);
    let mut out_file = BufWriter::new(File::create(output).context("create output")?);

    let passphrase = SecretString::new(passphrase.to_string().into_boxed_str());
    let identity = Identity::new(passphrase);
    let decryptor = Decryptor::new(&mut in_file).context("parse age header")?;
    let mut reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity)).context("decrypt")?;

    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).context("read decrypted")?;
        if n == 0 { break; }
        out_file.write_all(&buf[..n]).context("write output")?;
    }
    out_file.flush().context("flush output")?;
    Ok(())
}