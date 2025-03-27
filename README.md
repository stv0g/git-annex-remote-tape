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


## On-tape format

```
+=====================+
| Media Header        |
| - Version           |
| - Header Length     |
| - Create Time       |
+=====================+
|///// FILE MARK /////|
+=====================+
| Archive Header 1    |
| - Version           |
| - Header Length     |
| - Create Time       |
| - Host              |
|=====================|
| Object Header 1     |
| - Version           |
| - Header Length     |
| - Object Length     |
| - Key               |
|- - - - - - - - - - -|
| Object Contents     |
|---------------------|
| Object Header 2     |
  .....               |
+=====================+
|///// FILE MARK /////|
+=====================+
| Archive Header 2    |
|  .....              |
+=====================+
```
