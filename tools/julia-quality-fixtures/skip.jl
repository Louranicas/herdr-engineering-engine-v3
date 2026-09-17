# Required but unexecuted: Test records Broken(:skipped), not a pass.
@test_skip Base.PkgId(Base.require(Main, :HabitatAnalysis)).name == "HabitatAnalysis"
