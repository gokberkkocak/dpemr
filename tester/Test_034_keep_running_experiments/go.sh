$1 -c $2 edit --create-table --load ../simple.txt
timeout 20  $1 -c $2 run --freq 1 --jobs 4 --keep-running >> /dev/null
echo $?
$1 -c $2 show --stats
$1 -c $2 show --all
