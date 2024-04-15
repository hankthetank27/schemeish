(define (map ls fn)
  (if (null? ls)
    (nil)
    (cons (fn (car ls))
          (map (cdr ls) fn))))

(define x (cons 1 (cons 2 (cons 3 (nil)))))
(define y x)
(define m1 (map  y (lambda (x) (* x 2))))
(define m2 (map m1 (lambda (x) (* x 2))))

