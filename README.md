# schemeish #

A toy Scheme interperter based off the metacircular evaluator from SICP. Supports a good amount of the basic primitive and derived expression types (`define`, `if`/`cond`, `lambda`, `let`,`cons`/`car`/`cdr`, etc), and includes mutable variable and list operations, `set!`, `set-car!`, `set-cdr!`.

To try it out, run `cargo run` to compile and enter the REPL, or pass in a file path as a command line argument to evaluate.


### References: ###

* [SICP Chapter 4, Metalingustic Abstraction](https://web.mit.edu/6.001/6.037/sicp.pdf)
* [MIT Scheme Spec](https://groups.csail.mit.edu/mac/ftpdir/scheme-7.4/)
* [Revised Scheme Standard](https://standards.scheme.org/official/r7rs.pdf)
* [steel](https://github.com/mattwparas/steel)
* [scheme.rs](https://github.com/isamert/scheme.rs)

