# Guantanamo Bay: 19.9N, 75.1W
$lat = 19.9
$lon = -75.1

$lonField = [math]::Floor(($lon + 180) / 20)
$latField = [math]::Floor(($lat + 90) / 10)
$field1 = [char]([int][char]'A' + $lonField)
$field2 = [char]([int][char]'A' + $latField)
$lonSquare = [math]::Floor((($lon + 180) % 20) / 2)
$latSquare = [math]::Floor((($lat + 90) % 10) / 1)

"Guantanamo Bay (19.9N, 75.1W) = Grid: $field1$field2$lonSquare$latSquare"

# Typical US grids for comparison
"US East Coast starts with: FM, FN (Florida=EL, EM)"
"US Midwest: EM, EN"
