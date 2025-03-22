# git-annex-remote-tape

```shell
git annex initremote type=external externaltype=tape drive=/dev/st0
```

```shell
git-annex-remote-tape init <NAME>
git-annex-remote-tape repos
git-annex-remote-tape tapes
git-annex-remote pending --tape --repo
git-annex-remote-tape retrieve --tape --repo
```

```shell
~/.git-annex/remotes
    UUID -> ~/my-repo

~/my-repo/.git/annex/tapes
    UUID/
        details.json
```
