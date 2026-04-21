from __future__ import annotations

import unittest

from src.sss_blueprint import MVP_MILESTONES, render_sss_mvp_plan


class SssBlueprintTests(unittest.TestCase):
    def test_sss_mvp_plan_names_core_surfaces(self) -> None:
        plan = render_sss_mvp_plan()

        self.assertIn("Serial Alice Sky MVP Plan", plan)
        self.assertIn("Digital twin core", plan)
        self.assertIn("rust/crates/sss-core", plan)
        self.assertIn("Claw as the agent harness", plan)

    def test_sss_mvp_milestones_are_actionable(self) -> None:
        self.assertGreaterEqual(len(MVP_MILESTONES), 4)
        self.assertTrue(all(milestone.outcome for milestone in MVP_MILESTONES))
        self.assertTrue(all(milestone.implementation_surface for milestone in MVP_MILESTONES))


if __name__ == "__main__":
    unittest.main()
