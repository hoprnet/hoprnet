# Homebrew Formula

## Development

Those instructions are only at development time of the Formula

### Install

- Build hoprd for your architecture and place it in a temp folder
- Build hopli for your architecture and place it in a temp folder
- Change the url of the artifacts at `./deploy/Formula/hoprd.rb` to use the temporary artifact
```bash
brew install --verbose --debug ./deploy/Formula/hoprd.rb
brew reinstall --verbose --debug hoprd
brew postinstall hoprd
```
- Edit manually $(brew --prefix)/etc/hoprd/hoprd.env
- brew services start hoprd

### Uninstall

```bash
brew uninstall --force hoprd
```

- Clear cache
```bash
rm -rf $(brew --cache)/*hopr*
rm -rf $(brew --prefix)/Cellar/hoprd
rm -f $(brew --prefix)/bin/hopr{d,li}
rm -rf $(brew --prefix)/etc/hoprd
rm -rf $(brew --prefix)/var/lib/hoprd
rm -f $(brew --prefix)/var/log/hoprd.log
rm -f $(brew --prefix)/var/homebrew/linked/hoprd
brew cleanup -s
```
