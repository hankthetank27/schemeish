pub const PRELUDE: &str = "
;; some functions are from:
;; https://en.wikibooks.org/wiki/Write_Yourself_a_Scheme_in_48_Hours/Towards_a_Standard_Library

(define (id x) x)
(define (curry func arg1) (lambda (arg) (apply func (cons arg1 arg))))
(define (compose f g) (lambda (arg) (f (apply g arg))))

;; folds
(define (foldr func end lst)
  (if (null? lst)
      end
      (func (car lst) (foldr func end (cdr lst)))))

(define (foldl func accum lst)
  (if (null? lst)
      accum
      (foldl func (func accum (car lst)) (cdr lst))))

(define (unfold func init pred)
  (if (pred init)
      (cons init '())
      (cons init (unfold func (func init) pred))))

(define fold foldl)
(define reduce foldr)

;; numbers
(define zero? (curry = 0))
(define positive? (curry < 0))
(define negative? (curry > 0))
(define (odd? num)  (= (remainder num 2) 1))
(define (even? num) (= (remainder num 2) 0))
(define (abs num) (if (negative? num) (- num) num))
(define (gcd a b) (if (= b 0) (abs a) (gcd b (modulo a b))))
(define (lcm a b) (/ (abs (* a b)) (gcd a b)))

;; lists
(define (list-ref s i) (list-copy s i (+ i 1)))
(define (map func lst) (foldr (lambda (x y) (cons (func x) y)) '() lst))
(define (filter pred lst) (foldr (lambda (x y) (if (pred x) (cons x y) y)) '() lst))
(define (reverse lst) (fold (flip cons) '() lst))
(define (length lst) (fold (lambda (x y) (+ x 1)) 0 lst))
(define (list-tail lst n) (if (<= n 0) lst (list-tail (cdr lst) (- n 1))))
(define (list-head lst n) (if (<= n 0) '() (cons (car lst) (list-head (cdr lst) (- n 1)))))
(define (list-ref lst n) (car (list-tail lst n)))


(define (mem-helper pred op) (lambda (acc next) (if (and (not acc) (pred (op next))) next acc)))
(define (memq obj lst)       (fold (mem-helper (curry eq? obj) id) #f lst))
(define (memv obj lst)       (fold (mem-helper (curry eqv? obj) id) #f lst))
(define (member obj lst)     (fold (mem-helper (curry equal? obj) id) #f lst))
(define (assq obj alist)     (fold (mem-helper (curry eq? obj) car) #f alist))
(define (assv obj alist)     (fold (mem-helper (curry eqv? obj) car) #f alist))
(define (assoc obj alist)    (fold (mem-helper (curry equal? obj) car) #f alist))


(define (caar x) (car (car x)))
(define (cadr x) (car (cdr x)))
(define (cdar x) (cdr (car x)))
(define (cddr x) (cdr (cdr x)))
(define (caaar x) (car (car (car x))))
(define (caadr x) (car (car (cdr x))))
(define (cadar x) (car (cdr (car x))))
(define (caddr x) (car (cdr (cdr x))))
(define (cdaar x) (cdr (car (car x))))
(define (cdadr x) (cdr (car (cdr x))))
(define (cddar x) (cdr (cdr (car x))))
(define (cdddr x) (cdr (cdr (cdr x))))
(define (caaaar x) (car (car (car (car x)))))
(define (caaadr x) (car (car (car (cdr x)))))
(define (caadar x) (car (car (cdr (car x)))))
(define (caaddr x) (car (car (cdr (cdr x)))))
(define (cadaar x) (car (cdr (car (car x)))))
(define (cadadr x) (car (cdr (car (cdr x)))))
(define (caddar x) (car (cdr (cdr (car x)))))
(define (cadddr x) (car (cdr (cdr (cdr x)))))
(define (cdaaar x) (cdr (car (car (car x)))))
(define (cdaadr x) (cdr (car (car (cdr x)))))
(define (cdadar x) (cdr (car (cdr (car x)))))
(define (cdaddr x) (cdr (car (cdr (cdr x)))))
(define (cddaar x) (cdr (cdr (car (car x)))))
(define (cddadr x) (cdr (cdr (car (cdr x)))))
(define (cdddar x) (cdr (cdr (cdr (car x)))))
(define (cddddr x) (cdr (cdr (cdr (cdr x)))))
";
