#!/bin/bash

# Job Flags
#SBATCH -p mit_normal
#SBATCH -c 24
#SBATCH --mem=32GB
#SBATCH -t 00:15:00

cd ~/code20rust/newstuff
cargo r --release > out.wl 2>> log.txt