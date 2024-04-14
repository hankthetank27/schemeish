(define 1 2)

(define (product term a next b)
  (if (> a b)
      1
      (* (term a)
         (product term (next a) next b))))
(define (factorial x)
  (define (id x) x)
  (define (inc x) (+ x 1))
  (product id 1 inc x))
(factorial 10)

