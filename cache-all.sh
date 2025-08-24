#!/bin/bash

declare -a mensen
mensen=(
  academica
  ahornstrasse
  "bistro-templergraben"
  bayernallee
  "eupener-strasse"
  kmac
  suedpark
  vita
  juelich
)

for i in ${mensen[@]}; do
  mkdir -p ${i/-/_}
  acmensa-cli -m ${i} export -o ${i/-/_}
done
