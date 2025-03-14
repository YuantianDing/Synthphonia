; acs8/ZipCode.000002
(set-logic SLIA)
(define-fun f ((arg0 String)) String (list.at (str.split arg0 ",") -1.0))

(assert (= (f "Ak H     12 P Ave,Yangon,NY,(023) 966-2677,000-94-0933,14726") "14726"))
(check-sat)
