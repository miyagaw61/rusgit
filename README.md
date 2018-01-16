# rusgit - Git Wrapper in Rust

This is instead of https://github.com/miyagaw61/git2nd.
This is faster than git2nd.

### [!]This is under development now.
### [!]develop branch is the most newest.

# Install

## `Install cargo`

```Bash
curl https://sh.rustup.rs -sSf | sh #This command can download cargo to $HOME/.cargo
source $HOME/.cargo/env #You should write this to bashrc.
```

## `Install rusgit`


```Bash
cargo install --git https://github.com/miyagaw61/rusgit
```

## `You may become happy if you execute this command`


```Bash
#alias rusgit
alias rs="rusgit status"
alias ra="rusgit add"
alias rc="rusgit commit"
alias rac="rusgit ac"
alias rl="rusgit log"
alias rd="rusgit diff"
alias rb="rusgit branch"
alias rpush="rusgit push"
alias rpull="rusgit pull"
```

### Also if you execute this command, you can these alias.

```Bash
eval "$(rusgit alias)"
```
