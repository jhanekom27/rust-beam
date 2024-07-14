# rust-beam

**NOTE: This is still very unstable as in very early development**

`rust-beam` is a simple file transfer CLI tool.
Often at work it is needed to share passwords and secrets with others on your team,
but rarely does everyone have a file manager setup to easily share secrets with one another.
`rust-beam` lets you easily stream the file to another user via the relay server.
The file is never stored so there is no need to worry about how or where the data is used.

## Workflow

1. Sender uses `rust-beam` to send the file they want. `rb` will provide a UUID for the transfer
2. Sender passes the UUID to the receiver in any convenient way (e.g. slack or discord).
3. Receiver retrieves the file via the UUID using `rb`
4. The relay server facilitates streaming the data across allowing small and larger files to be sent easily without taxing the server.

## Installation

### Mac or windows

```bash
brew tap jhanekom27/rust-beam
brew install rust-beam
```

### Windows

¯\_(ツ)\_/¯

\_Not sure how to package for windows yet, but `.exe` is available via GitHub release

## Usage

### Send a file

`rust-beam send -f <example-file>`

Wait for UUID and send to receiver

### Receive a file

`rust-beam receive <UUID>`

Wait for file transfer to finish

## TODO List

- [ ] Allow file compression before sending to reduce transfer size
  - [ ] Do a speed test with this
- [ ] Allow file encryption before sending
- [x] Use a shorter UUID
- [x] Copy UUID to clipboard
- [ ] Improve info presentation during usage
- [ ] Allow retention of original file
- [ ] Allow renaming file when saving
- [x] Improve connection handling, no file transfer until receiver connected

### Potential updates

- [ ] Allow custom config for personal relay server
- [ ] Reduce container size
