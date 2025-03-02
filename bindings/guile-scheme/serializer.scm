;; pop-test - your test orchestration master of puppets

;; Copyright (C) 2025 Josep Bigorra (jjba23) <jjbigorra@gmail.com>

;; pop-test is free software: you can redistribute it and/or modify
;; it under the terms of the Lesser GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; pop-test is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; Lesser GNU General Public License for more details.

;; You should have received a copy of the Lesser GNU General Public License
;; along with pop-test.  If not, see <https://www.gnu.org/licenses/>.

(use-modules (json))
(use-modules (ice-9 string-fun))

(define (->pop-test-scenario obj)
  (cond
   ((pair? obj) ; Check if it's a cons cell (alist pair or list)
    (let ((key (car obj))
          (value (cdr obj)))
      (if (and (symbol? key) (pair? obj))
          (cons (string-replace-substring (symbol->string key) "-" "_") (->pop-test-scenario value)) 
          (cons (->pop-test-scenario key) (->pop-test-scenario value))))) 
   ((list? obj)
    (map ->pop-test-scenario obj))
   ((symbol? obj)
    (string-replace-substring (symbol->string obj) "-" "_"))
   ((vector? obj)
    (list->vector (map ->pop-test-scenario (vector->list obj))))
   (else obj)))

(define (main args)
  (if (< (length args) 3)
      (begin
        (display "Usage: guile serializer.scm <dest-filename>\n")
        (exit 1))
      (begin
        (display "loading scenario...")
        (load (cadr args))
        (let* ((filename (caddr args)))
          (display "writing to file...")
          (display filename)
          (newline)
          (call-with-output-file filename
            (lambda (port)
              (scm->json (->pop-test-scenario pop-test-scenario) port #:pretty #t)))))))

(main (command-line))
