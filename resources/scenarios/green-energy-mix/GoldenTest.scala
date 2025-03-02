// pop-test - your test orchestration master of puppets

// Copyright (C) 2025 Josep Bigorra (jjba23) <jjbigorra@gmail.com>

// pop-test is free software: you can redistribute it and/or modify
// it under the terms of the Lesser GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// pop-test is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// Lesser GNU General Public License for more details.

// You should have received a copy of the Lesser GNU General Public License
// along with pop-test.  If not, see <https://www.gnu.org/licenses/>.

import poptest.models._
import poptest.Serializer._

object GoldenTest extends App {

  val greenEnergyMixService = PopService(
    id = "green-energy-mix",
    name = None,
    dir = "$HOME/Ontwikkeling/Werk/Vandebron",
    baseUrl = "http://localhost:50000",
    startCmdArgs = Seq("sbt \"greenEnergyMix/run\""),
    envVars = Some(
      Map(
        "NATIONAAL_ENERGIE_DASHBOARD_TOKEN" -> "002b9a99a4d98cdee8aabf3abb0778cb649126ffbffc1bcde516884797809ce9"
      )
    ),
    healthCheck = Http(
      path = "/health",
      patienceSeconds = 120
    )
  )

  val popScenario = PopScenario(
    popServices = Some(
      Seq(
        greenEnergyMixService
      )
    ),
    popActions = Some(
      Seq(
        HttpAssertionAction(
          name = Some(
            "Check that the forecast endpoint returns a 200 with correct data, with load and solar"
          ),
          assertUrl = "/api/v1/forecast",
          serviceId = "green-energy-mix",
          method = "GET",
          jsonPatterns = Some(
            Seq(
              BodyPattern(
                pattern = "meta.forecastDate",
                condition = "non-empty-str"
              ),
              BodyPattern(
                pattern = "data.[0].load",
                condition = "> 0"
              ),
              BodyPattern(
                pattern = "data.[0].solar",
                condition = ">= 0"
              )
            )
          )
        ),
        HttpAssertionAction(
          name = Some(
            "Check that the window endpoint returns a 200 with correct data"
          ),
          assertUrl = "/api/v1/window",
          serviceId = "green-energy-mix",
          method = "GET",
          jsonPatterns = Some(
            Seq(
              BodyPattern(
                pattern = "greenPercentage",
                condition = ">= 0"
              ),
              BodyPattern(
                pattern = "windowStartUtc",
                condition = "non-empty-str"
              ),
              BodyPattern(
                pattern = "windowStartAms",
                condition = "non-empty-str"
              )
            )
          )
        )
      )
    )
  )

  serializeToFile(args, popScenario)
}
