(use-modules (json))
(use-modules (ice-9 string-fun))

(define (alist-keys->strings obj)
  (cond
   ((pair? obj) ; Check if it's a cons cell (alist pair or list)
    (let ((key (car obj))
          (value (cdr obj)))
      (if (and (symbol? key) (pair? obj))
          (cons (string-replace-substring (symbol->string key) "-" "_") (alist-keys->strings value)) ; Convert key and recurse
          (cons (alist-keys->strings key) (alist-keys->strings value))))) ; Recurse on both key and value
   ((list? obj) ; Check if it's a list (handle nested lists)
    (map alist-keys->strings obj))
   ((symbol? obj) ; Convert bare symbols to strings
    (string-replace-substring (symbol->string obj) "-" "_"))
   (else obj)))


(alist-keys->strings wikimusic-service)
(alist-keys->strings wikimusic-check-welcome)

(display (scm->json (alist-keys->strings pop-test-scenario)))
