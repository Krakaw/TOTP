# TOTP

A CLI tool for managing TOTP accounts.
All accounts are stored in a local file that's encrypted with the provided password.
If you don't provide the -p argument it will expect the password on stdin.


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

### 
