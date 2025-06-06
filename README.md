# TermsLang
The terms programming language

> Simple, Concise, ~~Fast~~

## Latest Version: `v0.8.6`

## Example
```swift
"
My FizzBuzz Program
Created Fri Sep 6, 2024

FizzBuzz test program created for practice.
- No command line arguments
- No dependencies
"

struct FizzBuzz {
    let int iters ~

    let int bob ~

    func null @new: int iters {
        updt @this.iters = iters ~
    }

    func int run: str _3Word, str _5Word {
        loop idx: idx < @this.iters {
            let str result = "" ~
            let int idx = idx + 1 ~

            if idx % 3 == 0 {
                updt result += _3Word ~
            }

            if idx % 5 == 0 {
                updt result += _5Word ~
            }

            if result == "" {
                updt result += idx.@str.() ~
            }

            println result ~
        }

        return @this.iters ~
    }
}

func null printResults: int iters {
    println "FizzBuzz Program Complete %/% iterations run." % iters.@str.() ~
}

func null @main: str[] args {
    println "Hello World" ~

    let FizzBuzz fizzy = $(1) FizzBuzz ~
    let int iters = fizzy.run.("Fizz", "Buzz") ~
    cll printResults.(iters) ~
}
```

## Installation
```
cargo install termslang
```

## Run File
```
termslang run example.tms
```

## Format File
```
termslang format example.tms
```

## Debug File
```
termslang debug example.tms
```

## Supports
VSCode Support: https://github.com/Owen-Dechow/TermsVsCodeSupport
