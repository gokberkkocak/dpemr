$1 -c $2 edit --create-table --load ../simple_2.txt
$1 -c $2 run --freq 1 >> /dev/null
$1 -c $2 show --stats
$1 -c $2 show --all
