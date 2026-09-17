# --depwarn=error must turn this explicit deprecation into a test error.
@test Base.depwarn("HEE3_EXPECTED_DEPRECATION_CONTROL", :hee3_fixture; force = true) ===
      nothing
