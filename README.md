# crypt-tool

Ein natives Linux-Programm zum Ver- und Entschlüsseln von Dateien mit einer grafischen Oberfläche und einer Kommandozeile.

Die Verschlüsselung basiert auf dem [age](https://age-encryption.org/)-Format mit einer Passphrase. Die Anwendung verwendet etablierte Kryptografie-Bibliotheken und implementiert keine eigenen Verschlüsselungsalgorithmen.

## Funktionen

- Grafische Oberfläche für Linux
- Kommandozeilenprogramm für Skripte und Automatisierung
- Verschlüsselung und Entschlüsselung einzelner Dateien
- Passphrase-basierte Verschlüsselung
- Streaming-Verarbeitung in 64-KB-Blöcken
- Keine temporären Klartextdateien
- Ausgabe im `.age`-Format
- Statisches CLI-Binary für Linux x86_64
- GUI-Binary für Linux x86_64

## Schnellstart mit fertigen Binaries

Die fertigen Binaries befinden sich auf der [Release-Seite](https://github.com/NoggerCooker/crypt-tool/releases).

### GUI

```bash
chmod +x crypt-tool-gui-linux-x86_64
./crypt-tool-gui-linux-x86_64
```

Die GUI bietet:

1. **Verschlüsseln** oder **Entschlüsseln** auswählen
2. Eingabedatei auswählen
3. Ausgabedatei prüfen oder ändern
4. Passphrase zweimal eingeben
5. Vorgang starten

### CLI

Binary ausführbar machen:

```bash
chmod +x crypt-tool-cli-linux-x86_64
```

Datei verschlüsseln:

```bash
./crypt-tool-cli-linux-x86_64 encrypt datei.txt --passphrase 'Meine Passphrase'
```

Das erzeugt standardmäßig:

```text
datei.age
```

Datei entschlüsseln:

```bash
./crypt-tool-cli-linux-x86_64 decrypt datei.age --passphrase 'Meine Passphrase'
```

Ohne `--passphrase` fragt das Programm die Passphrase interaktiv ab:

```bash
./crypt-tool-cli-linux-x86_64 encrypt datei.txt
./crypt-tool-cli-linux-x86_64 decrypt datei.age
```

Die interaktive Eingabe ist vorzuziehen, weil die Passphrase dann nicht in der Shell-History oder Prozessliste auftaucht.

## Voraussetzungen zum Bauen

Für die CLI:

- Linux x86_64
- Rust über `rustup`
- `musl-tools`
- `build-essential`

Für die GUI zusätzlich:

- `pkg-config`
- GTK3- und GLib-Entwicklungsbibliotheken

Auf Debian oder Ubuntu:

```bash
sudo apt update
sudo apt install -y build-essential musl-tools pkg-config libgtk-3-dev libglib2.0-dev
```

Rust installieren, falls es noch nicht vorhanden ist:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Projekt bauen

Projekt herunterladen:

```bash
git clone https://github.com/NoggerCooker/crypt-tool.git
cd crypt-tool
```

### CLI-Binary bauen

Für ein statisch gelinktes Linux-x86_64-Binary:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl --bin crypt-tool
```

Das Binary liegt anschließend hier:

```text
target/x86_64-unknown-linux-musl/release/crypt-tool
```

Optional kopieren und umbenennen:

```bash
cp target/x86_64-unknown-linux-musl/release/crypt-tool crypt-tool-cli-linux-x86_64
```

### GUI-Binary bauen

```bash
cargo build --release --features gui --bin crypt-tool-gui
```

Das Binary liegt anschließend hier:

```text
target/release/crypt-tool-gui
```

Optional kopieren und umbenennen:

```bash
cp target/release/crypt-tool-gui crypt-tool-gui-linux-x86_64
```

Die GUI wird derzeit dynamisch gegen die Linux-GUI-Bibliotheken gelinkt. Auf dem Zielsystem müssen daher die üblichen GTK-/GLib-Laufzeitbibliotheken vorhanden sein. Das CLI-Binary wird statisch gegen musl gebaut und benötigt keine Rust-Laufzeitinstallation.

## Projektstruktur

```text
crypt-tool/
├── Cargo.toml                 # Rust-Projekt und Abhängigkeiten
├── Cargo.lock                 # Fixierte Abhängigkeitsversionen
├── src/
│   ├── main.rs                # CLI
│   └── gui.rs                 # GUI
├── crypt-tool                 # vorhandenes CLI-Binary
├── crypt-tool-bin             # CLI-Binary
├── crypt-tool-gui             # GUI-Binary
└── README.md                  # diese Dokumentation
```

Der Ordner `target/` wird nicht versioniert. Er enthält nur generierte Build-Zwischendateien und kann jederzeit neu erzeugt werden.

## Kryptografie und Dateiformat

- Dateiformat: `age`
- Passphrase-Modus: scrypt-basierte Schlüsselableitung
- Authenticated Encryption: ChaCha20-Poly1305 innerhalb des age-Formats
- Verarbeitung: Streaming mit 64-KB-Puffern
- Falsche Passphrasen oder beschädigte Dateien führen zu einem Fehler

Die Passphrase ist nicht wiederherstellbar. Wenn sie verloren geht, können die verschlüsselten Daten nicht entschlüsselt werden.

## Prüfsummen der Release-Binaries

Zu jedem Release gehört eine Datei namens `SHA256SUMS-<version>.txt`. Damit kann geprüft werden, ob ein heruntergeladenes Binary unverändert ist:

```bash
sha256sum -c SHA256SUMS-v0.1.0.txt
```

Die Prüfung funktioniert, wenn sich die Prüfsummendatei und die beiden Binaries im selben Ordner befinden.

## Versionen und Releases

Das Projekt verwendet Versionsnummern nach dem Schema:

```text
MAJOR.MINOR.PATCH
```

- `PATCH`: Fehlerbehebungen, zum Beispiel `v0.1.1`
- `MINOR`: neue rückwärtskompatible Funktionen, zum Beispiel `v0.2.0`
- `MAJOR`: inkompatible Änderungen, zum Beispiel `v1.0.0`

Die Versionsnummer in `Cargo.toml`, der Git-Tag, der GitHub-Release und die Release-Dateinamen sollten übereinstimmen.

Aktuelle Version:

- [v0.1.0 auf GitHub](https://github.com/NoggerCooker/crypt-tool/releases/tag/v0.1.0)

## Lizenz

Derzeit ist noch keine separate Lizenzdatei festgelegt. Vor einer öffentlichen Weitergabe sollte eine Lizenz, zum Beispiel MIT oder Apache-2.0, ausdrücklich ergänzt werden.
