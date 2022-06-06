# TOTP

A CLI tool for managing TOTP accounts.
All accounts are stored in a local file that's encrypted with the provided password.
If you don't provide the -p argument it will expect the password on stdin.

Running `totp` without any arguments will run the TUI interface.

## Usage

```bash
totp --help
totp 0.1.0
Krakaw <41575888+Krakaw@users.noreply.github.com>
Generate TOTP codes

USAGE:
    totp [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -f, --filename <FILENAME>    The storage filename [default: .storage.txt]
    -h, --help                   Print help information
    -p, --password <PASSWORD>    The encryption password
    -V, --version                Print version information

SUBCOMMANDS:
    add         Add a new account
    check       Check an OTP
    delete      Delete an account
    generate    Generate an OTP
    help        Print this message or the help of the given subcommand(s)
```

### Add accounts

    totp -p password add -a AccountName -s SecretToken

### Generate tokens in a loop

    totp -p password generate -r

    Account1   123456     04
    Account2   123456     04
    Account3   123456     04

### Delete an account

    totp -p password delete -a AccountName

### Check an OTP against a secret for a specific time within a range

    totp -p password check -t TokenSecretKey -o 123456 -s 2022-06-03T08:35:00+02:00 -r 10  

## Key Bindings

### User Interface

#### Global Key Bindings
| Key Binding | Action                  |
|-------------|-------------------------|
| `/`         | Switch to insert mode   |
| `Esc`       | Switch to normal mode   |
| `Ctrl-c`    | Exit                    |
| `Down`      | Select next account     |
| `Up`        | Select previous account |
| `Enter`     | Copy OTP to clipboard   |
