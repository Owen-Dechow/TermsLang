# TermsLang
The terms programming language

> Simple, Consise, Fast

## Latest Version: `v0.0`

## Example

```
class FizzBuzz: {
    func null @new: {
        println "FizzBuzz Object Created";
    }

    func null run: int n {
        loop i: i < n {
            let str result = "";

            if i % 3 == 0 {
                updt result += "Fizz";
            }

            if i % 5 == 0 {
                updt result += "Buzz";
            }

            if result == "" {
                println i;
                return null;
            }

            println result;
        }
    }
}

func null main: str[] args {
    let FizzBuzz fizzy = $() FizzBuzz;
    cll fizzy.run.(100);
}
```

## Compiler
- [x] Main Lexer
- [x] Main Parser
- [x] Temperary Transpiler
- [ ] Main LLVM Compiler
- [ ] Version 1.0

Main implimentations are not fully complete for `v1.0` goals see [Still in development](still-in-development)

  
The goal of `TermsLang` is to be compiled using LLVM the `terms2rust_transpiler` is only tempary and fails to properly transpile most code.

## Still in development
* String escape sequences
* Better error messages
* Standard library