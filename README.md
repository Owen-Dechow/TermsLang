# TermsLang
The terms programming language

> Simple, Consise, Fast

## Latest Version: `v0.0`

## Example
Depending on `.md` renderer example may not display color.
<pre color="darkgrey">
<code>
<span style="color: grey"># Custom Struct</span>
<span style="color: orchid">struct</span> MyCustomStruct {}

<span style="color: grey"># Declare a Function</span>
<span style="color: orchid">func</span> null do_a_thing<span style="color: tan">:</span> {
    <span style="color: grey"># Get user name</span>
    <span style="color: orchid">print</span> <span style="color: #FF4500">"What is your name? "</span>
    <span style="color: orchid">let</span> str name = <span style="color: #FF4500">""</span><span style="color: tan">;</span>
    <span style="color: orchid">readln</span> name<span style="color: tan">;</span>
    <span style="color: orchid">println</span> f<span style="color: #FF4500">"Hello, {name}"</span>

    <span style="color: grey"># Create loop</span>
    <span style="color: orchid">loop</span> i: i <span style="color: tan"><</span> 100 {
        <span style="color: orchid">print</span> <span style="color: #FF4500">"Number: {i}"</span><span style="color: tan">;</span>
        
        <span style="color: orchid">if</span> i <span style="color: tan"><</span> 10 {
            <span style="color: orchid">continue</span><span style="color: tan">;</span>
        } <span style="color: orchid">else</span> {
            <span style="color: orchid">break</span><span style="color: tan">;</span>
        }
    }

    <span style="color: orchid">updt</span> name = <span style="color: #FF4500">"My New Value"</span><span style="color: tan">;</span>
    <span style="color: orchid">println</span> name<span style="color: tan">;</span>
}

<span style="color: grey"># main function</span>
<span style="color: orchid">func</span> null main: str[] args {

    <span style="color: grey"># Create MyCustomStruct Instance</span>
    <span style="color: orchid">let</span> MyCustomStruct my_instance = $() MyCustomStruct<span style="color: tan">;</span>

    <span style="color: grey"># Call a Function</span>
    <span style="color: orchid">cll</span> do_a_thing.(100)<span style="color: tan">;</span>
}
</code>
</pre>


## Compiler
- [x] Main Lexer
- [x] Main Parser
- [ ] Main Active Parser
- [ ] Version 1.0