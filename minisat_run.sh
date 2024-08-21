#!/bin/sh

while :; do
  minisat solutions/8-minisat-iter2.txt 8.txt
#   cat 8.txt
  if [ `head -1 8.txt` = UNSAT ]; then
    break
  fi
  tail -1 8.txt |
    awk '{
      for(i=1;i<NF;++i) { $i = -$i }
      print
    }' >> solutions/8-minisat-iter2.txt
done