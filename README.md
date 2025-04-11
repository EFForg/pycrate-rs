# pycrate-rs

This repo contains two projects: a parser generator that uses [pycrate](https://github.com/pycrate-org/pycrate/) to generate Rust code that parses telcom data, and the Rust crate itself.

## Generating the parser/tests

From a clean repo:

```
$ python -m venv venv
$ . venv/bin/activate
$ pip install ./generator-script
$ python generator-script/main.py src/nas/generated
$ cargo test
```

If you have a directory of pcaps you'd like to generate tests from, pass its path as another argument to `generator-script/main.py`.
