# clazydbm

A modern Terminal User Interface (TUI) database management tool for PostgreSQL, MySQL, and SQLite.

## Features

- **Multi-database support**: PostgreSQL, MySQL, and SQLite
- **Terminal User Interface**: Clean, keyboard-driven interface built with Ratatui
- **Database browsing**: Navigate through databases, tables, and schemas
- **Data viewing**: Browse table records with pagination and horizontal scrolling
- **Table properties**: View column information, data types, and constraints
- **External CLI integration**: Seamlessly launch pgcli, mycli, or litecli for advanced SQL operations
- **Connection management**: Save and manage multiple database connections
- **Real-time search**: Filter databases and tables with live search functionality

## Installation

### From source

```bash
git clone https://github.com/your-username/clazydbm.git
cd clazydbm
cargo install --path .
```

### Prerequisites

For SQL tab functionality, install the appropriate CLI tools:

```bash
# PostgreSQL
pip install pgcli

# MySQL
pip install mycli

# SQLite
pip install litecli
```

## Usage

### Basic Usage

```bash
clazydbm
```

### Configuration

On first run, clazydbm will create a configuration directory in your home folder. Edit the connections file to add your database connections:

**Linux/macOS**: `~/.config/clazydbm/connections.yaml`
**Windows**: `%APPDATA%/clazydbm/connections.yaml`

Example configuration:

```yaml
connections:
  - name: "Local PostgreSQL"
    type: postgres
    host: localhost
    port: 5432
    user: postgres
    password: password
    database: mydb

  - name: "Local MySQL"
    type: mysql
    host: localhost
    port: 3306
    user: root
    password: password
    database: mydb

  - name: "SQLite Database"
    type: sqlite
    path: "./example.db"
```

### Keyboard Shortcuts

#### Navigation
- `Tab` / `Shift+Tab`: Switch between panels
- `↑↓` / `jk`: Navigate lists
- `Enter`: Select item / Open table
- `Esc`: Go back

#### Table View
- `1`: Focus on Records tab
- `2`: Focus on SQL tab  
- `3`: Focus on Properties tab
- `←→` / `hl`: Scroll columns horizontally
- `[]`: Jump 5 columns left/right
- `Ctrl+A` / `Ctrl+E`: Jump to first/last column
- `PgUp` / `PgDn`: Scroll rows vertically
- `Home` / `End`: Jump to top/bottom

#### SQL Tab
- `Enter`: Launch external CLI tool (pgcli/mycli/litecli)

#### General
- `Ctrl+C`: Quit application

## Architecture

clazydbm is built with a modular architecture:

- **Database abstraction**: Clean trait-based interface for different database types
- **Component-based UI**: Reusable UI components with message-passing architecture  
- **Terminal management**: Proper terminal suspension/restoration for external tools
- **Asynchronous operations**: Non-blocking database operations

## Supported Databases

| Database | Status | CLI Integration |
|----------|--------|-----------------|
| PostgreSQL | ✅ | pgcli |
| MySQL | ✅ | mycli |
| SQLite | ✅ | litecli |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details
