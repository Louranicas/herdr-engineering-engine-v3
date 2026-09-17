# --check-bounds=yes must override @inbounds and expose this BoundsError.
@test (@inbounds [11][2]) == 11
