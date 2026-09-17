# A passing package-load assertion must not hide a structured warning.
@test Base.PkgId(Base.require(Main, :HabitatAnalysis)).name == "HabitatAnalysis"
@warn "HEE3_EXPECTED_WARNING_CONTROL"
