import poptest.models._
import poptest.Serializer._

object GoldenTest extends App {

  val popScenario = PopScenario(
    popServices = Some(Seq(
      PopService(
        id = "green-energy-mix",
        name = None,
        dir = "$HOME/Ontwikkeling/Werk/Vandebron",
        baseUrl = "http://localhost:50000",
        shellPath = None,
        startCmdArgs = Seq("sbt \"greenEnergyMix/run\""),
        envVars = Some(Map(
          "NATIONAAL_ENERGIE_DASHBOARD_TOKEN" -> "002b9a99a4d98cdee8aabf3abb0778cb649126ffbffc1bcde516884797809ce9"
        )),
        healthCheck = Http(
          path = "/health",
          patienceSeconds = 120
        ),
        postgres = None,
        kafka = None,
        popServer = None,
        keycloak = None
      )
    )),
    popActions = Some(Seq(
      HttpAssertionAction(
        name = Some("Check that the forecast endpoint returns a 200 with correct data, with load and solar"),
        delaySeconds = None,
        assertCode = None,
        assertUrl = "/api/v1/forecast",
        serviceId = "green-energy-mix",
        method = "GET",
        jsonPatterns = Some(Seq(
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
        )),
        htmlPatterns = None,
        withKeycloakUserToken = None,
        messageFilePath = None,
        message = None,
        extraHeaders = None
      ),
      HttpAssertionAction(
        name = Some("Check that the window endpoint returns a 200 with correct data"),
        delaySeconds = None,
        assertCode = None,
        assertUrl = "/api/v1/window",
        serviceId = "green-energy-mix",
        method = "GET",
        jsonPatterns = Some(Seq(
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
        )),
        htmlPatterns = None,
        withKeycloakUserToken = None,
        messageFilePath = None,
        message = None,
        extraHeaders = None
      )
    )),
    popPostgres = None,
    popKafka = None,
    popServer = None,
    popKeycloak = None,
    popSqlite = None
  )

  serializeToFile(args, popScenario)
}
