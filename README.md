# TermsLang
The terms programming language

> Simple, Consise, Fast

## Latest Version: `v0.0`

## Example
<pre>
<span style="color:slategray;font-style:italic"># FizzBuzz class</span>
<span style="color:mediumorchid">class</span> <span style="color:tan">FizzBuzz</span>: {

    <span style="color:slategray;font-style:italic"># FizzBuzz initializer</span>
    <span style="color:mediumorchid">func</span> <span style="color:steelblue">null</span> <span style="color:tan">@new</span>: {
        <span style="color:mediumorchid">println</span> <span style="color:#03C03C">"FizzBuzz Object Created"</span>;
    }

    <span style="color:slategray;font-style:italic"># Run function</span>
    <span style="color:mediumorchid">func</span> <span style="color:steelblue">null</span> <span style="color:tan">run</span>: <span style="color:steelblue"><span style="color:goldenrod">int</span></span> <span style="color:tan">n</span> {
        <span style="color:slategray;font-style:italic"># Loop n times</span>
        <span style="color:mediumorchid">loop</span> <span style="color:tan">i</span>: <span style="color:tan">i</span> < <span style="color:tan">n</span> {

            <span style="color:slategray;font-style:italic"># Declare result var</span>
            <span style="color:mediumorchid">let</span> <span style="color:steelblue">str</span> <span style="color:tan">result</span> = <span style="color:#03C03C">""</span>;

            <span style="color:mediumorchid">if</span> <span style="color:tan">i</span> % <span style="color:goldenrod">3</span> == <span style="color:goldenrod">0</span> {
                <span style="color:slategray;font-style:italic"># Update result var</span>
                <span style="color:steelblue">updt</span> <span style="color:tan">result</span> += <span style="color:#03C03C">"Fizz"</span>;
            }

            <span style="color:mediumorchid">if</span> <span style="color:tan">i</span> % <span style="color:goldenrod">5</span> == <span style="color:goldenrod">0</span> {
                <span style="color:slategray;font-style:italic"># Update result var</span>
                <span style="color:steelblue">updt</span> <span style="color:tan">result</span> += <span style="color:#03C03C">"Buzz"</span>;
            }

            <span style="color:slategray;font-style:italic"># Check if no Fizz or Buzz</span>
            <span style="color:mediumorchid">if</span> <span style="color:tan">result</span> == <span style="color:#03C03C">""</span> {
                <span style="color:slategray;font-style:italic"># Print number (i)</span>
                <span style="color:mediumorchid">println</span> <span style="color:goldenrod"><span style="color:tan">i</span></span>;
                
                <span style="color:slategray;font-style:italic"># Exit function early</span>
                <span style="color:mediumorchid">return</span> <span style="color:steelblue">null</span>;
            }

            <span style="color:slategray;font-style:italic"># Print result</span>
            <span style="color:mediumorchid">println</span> <span style="color:tan">result</span>;
        }
    }
}

<span style="color:slategray;font-style:italic"># main function</span>
<span style="color:mediumorchid">func</span> <span style="color:steelblue">null</span> <span style="color:tan">main</span>: <span style="color:steelblue">str</span>[] <span style="color:tan">args</span> {

    <span style="color:slategray;font-style:italic"># Create FizzBuzz instance</span>
    <span style="color:steelblue">let</span> <span style="color:tan">FizzBuzz</span> <span style="color:tan">fizzy</span> = $() <span style="color:tan">FizzBuzz</span>;

    <span style="color:slategray;font-style:italic"># Call run function</span>
    <span style="color:steelblue">cll</span> <span style="color:tan">fizzy</span>.<span style="color:tan">run</span>.(<span style="color:goldenrod">100</span>);
}
</pre>

## Compiler
- [x] Main Lexer
- [x] Main Parser
- [ ] Main LLVM Compiler
- [ ] Version 1.0

Main implimentations are not fully complete for `v1.0` goals see [Still in development](still-in-development)

The goal of `TermsLang` is to be compiled using LLVM any interpreter or transpiler is temperary and to be used for testing.

## Still in development
* String escape sequences
* Better error messages
* Standard library