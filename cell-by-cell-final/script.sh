#!/bin/bash

# Job Flags
#SBATCH -p mit_normal
#SBATCH -c 32
#SBATCH --mem=16GB
#SBATCH -t 02:00:00

cd ~/code20rust/cell-by-cell-final