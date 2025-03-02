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

(define wikimusic-service
  `((id . "wikimusic") (dir . "$HOME/Ontwikkeling/Persoonlijk/wikimusic")
    (base-url . "http://localhost:50050")
    (start-cmd-args "guix shell -m manifest.scm -- art work")
    (health-check (path . "/")
                  (patience-seconds . 30))))

(define wikimusic-check-welcome
  `((name . "Check that the welcome page works well") (assert-url . "/")
    (method . "GET")
    (assert-code . 200)
    (service-id . "wikimusic")
    (html-patterns "h2")))

(define pop-test-scenario
  `((pop-services unquote
                  (list wikimusic-service))
    (pop-actions unquote
                 (list wikimusic-check-welcome))))

;;

