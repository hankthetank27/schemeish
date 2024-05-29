# schemeish #

A tiny Scheme interperter based off the metacircular evaluator from SICP. Supports a good amount of the basic primitive and derived expression types, and includes mutable variable and list operations.

To try it out, run `cargo run` to compile and enter the REPL, or pass in a file path as a command line argument to evaluate.

### Supported Operations ###

|                               |                     |
| ----------------------------- | ------------------- |
| **Primitives/Special Forms**  | **Libray**          |
| `lambda`                      | `id`                |
| `define`                      | `curry`             |
| `quote` / `'`                 | `compose`           |
| `if`                          | `foldr` / `reduce`  |
| `cond`                        | `foldl` / `fold`    |
| `begin`                       | `unfold`            |
| `let`                         | `zero?`             |
| `let*`                        | `positive?`         |
| `and`                         | `negitive?`         |
| `or`                          | `odd?`              |
| `not`                         | `even?`             |
| `+`                           | `abs`               |
| `-`                           | `map`               |
| `*`                           | `filter`            |
| `/`                           | `length`            |
| `=`                           | `list-tail`         |
| `>`                           | `list head`         |
| `<`                           | `memq`              |
| `<=`                          | `memv`              |
| `>=`                          | `member`            |
| `remainder`                   | `assq`              |
| `modulo`                      | `assv`              |
| `apply`                       | `assoc`             |
| `cons`                        | `caar`              |
| `car`                         | `cadr`              |
| `cdr`                         | `cdar`              |
| `list`                        | `cddr`              |
| `set!`                        | `caaar`             |
| `set-car!`                    | `caadr`             |
| `set-cdr!`                    | `cadar`             |
| `display`                     | `caddr`             |
| `error`                       | `cdaar`             |
| `equal?`                      | `cdadr`             |
| `eq?`                         | `cddar`             |
| `number?`                     | `cdddr`             |
| `symbol?`                     | `caaaar`            |
| `string?`                     | `caaadr`            |
| `pair?`                       | `caadar`            |
| `null?`                       | `caaddr`            |
|                               | `cadaar`            |
|                               | `cadadr`            |
|                               | `caddar`            |
|                               | `cadddr`            |
|                               | `cdaaar`            |
|                               | `cdaadr`            |
|                               | `cdadar`            |
|                               | `cdaddr`            |
|                               | `cddaar`            |
|                               | `cddadr`            |
|                               | `cdddar`            |
|                               | `cddddr`            |
|                               
                                
                                
                                
### References: ###

* [SICP Chapter 4, Metalingustic Abstraction](https://web.mit.edu/6.001/6.037/sicp.pdf)
* [MIT Scheme Spec](https://groups.csail.mit.edu/mac/ftpdir/scheme-7.4/)
* [Revised Scheme Standard](https://standards.scheme.org/official/r7rs.pdf)
* [steel](https://github.com/mattwparas/steel)
* [scheme.rs](https://github.com/isamert/scheme.rs)

