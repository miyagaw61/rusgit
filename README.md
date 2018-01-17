# rusgit - Git Wrapper in Rust

[![Twitter](https://imgur.com/Ibo0Twr.png)](https://twitter.com/miyagaw61)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](http://choosealicense.com/licenses/mit/)

fast and efficient Git Wrapper  
https://qiita.com/miyagaw61/items/893fd5a2115d0d2429de

# Install

### Install cargo

```Bash
curl https://sh.rustup.rs -sSf | sh #This command can download cargo to $HOME/.cargo
source $HOME/.cargo/env #You should write this to bashrc.
```

### Install rusgit


```Bash
cargo install --git https://github.com/miyagaw61/rusgit
```

### You may become happy if you execute this command


```Bash
alias rs="rusgit status"
alias ra="rusgit add"
alias rc="rusgit commit"
alias rac="rusgit ac"
alias rl="rusgit log"
alias rd="rusgit diff"
alias rb="rusgit branch"
alias ru="rusgit undo"
alias rt="rusgit tag"
alias rpush="rusgit push"
alias rpull="rusgit pull"
alias rmerge="rusgit merge"
alias rrebase="rusgit rebase"
alias rclone="rusgit clone"
```

If it is the same alias as above, you can also set it with the following command.

```Bash
eval "$(rusgit alias)"
```

# Usage

https://qiita.com/miyagaw61/items/893fd5a2115d0d2429de
