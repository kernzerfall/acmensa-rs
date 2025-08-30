#!/bin/bash
# Exports all available menus for all Mensen into subfolders (from cwd)
# (intended to be used with systemd timers for caching)

unset mensen
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
