use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::PathBuf;
use age::{Encryptor, Decryptor, scrypt::Identity, secrecy::SecretString};
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(name = "crypt-tool", version, about = "Simple file encryption with age")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encrypt a file
    Encrypt {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        /// Passphrase (will be prompted if omitted)
        #[arg(short, long)]
        passphrase: Option<String>,
        /// Output file (default: <input>.age)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Decrypt a file
    Decrypt {
        /// Input file (.age)
        #[arg(value_name = "FILE")]
        input: PathBuf,
        /// Passphrase (will be prompted if omitted)
        #[arg(short, long)]
        passphrase: Option<String>,
        /// Output file (default: input without .age)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn read_passphrase(prompt: &str) -> Result<Zeroizing<String>> {
    use std::io::{stdin, stdout, Write};
    print!("{}", prompt);
    stdout().flush()?;
    let mut pass = Zeroizing::new(String::new());
    stdin().read_line(&mut pass)?;
    Ok(Zeroizing::new(pass.trim_end_matches(&['\n', '\r'][..]).to_string()))
}

fn encrypt_file(input: &PathBuf, output: &PathBuf, passphrase: &str) -> Result<()> {
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

fn decrypt_file(input: &PathBuf, output: &PathBuf, passphrase: &str) -> Result<()> {
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Encrypt { input, passphrase, output } => {
            let pass = passphrase.unwrap_or_else(|| {
                read_passphrase("Passphrase: ").expect("read passphrase").to_string()
            });
            let output = output.unwrap_or_else(|| input.with_extension("age"));
            encrypt_file(&input, &output, &pass)?;
            eprintln!("✓ Encrypted: {}", output.display());
        }
        Command::Decrypt { input, passphrase, output } => {
            let pass = passphrase.unwrap_or_else(|| {
                read_passphrase("Passphrase: ").expect("read passphrase").to_string()
            });
            let output = output.unwrap_or_else(|| {
                let mut p = input.clone();
                p.set_extension("");
                p
            });
            decrypt_file(&input, &output, &pass)?;
            eprintln!("✓ Decrypted: {}", output.display());
        }
    }
    Ok(())
}