# Migrating to a monorepo

### Adding a repository into the monorepo

```
git subtree add --prefix=packages/website-hoprnet-org git@github.com:hoprnet/website-hoprnet-org.git master
```

### Pulling repository changes into the monorepo

```
git subtree pull --prefix=packages/website-hoprnet-org git@github.com:hoprnet/website-hoprnet-org.git master
```
