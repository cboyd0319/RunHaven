from __future__ import annotations

import unittest

from runhaven.images import build_image_plan
from runhaven.profiles import get_profile


class ImagePlanTests(unittest.TestCase):
    def test_custom_tag_is_used_for_build(self) -> None:
        plan = build_image_plan(get_profile("shell"), tag="runhaven/test:0.1.0")

        self.assertEqual(plan.command[3], "runhaven/test:0.1.0")

    def test_rejects_unsafe_tag(self) -> None:
        with self.assertRaisesRegex(ValueError, "image tag"):
            build_image_plan(get_profile("shell"), tag="--debug")


if __name__ == "__main__":
    unittest.main()
