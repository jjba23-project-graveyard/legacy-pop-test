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

//> using scala 2.13
//> using dep io.circe::circe-core:0.14.10
//> using dep io.circe::circe-generic:0.14.10
//> using dep io.circe::circe-generic-extras:0.14.4
//> using dep io.circe::circe-parser:0.14.10

package poptest.models

import io.circe.Codec
import io.circe.generic.extras.Configuration
import io.circe.generic.extras.semiauto._
import io.circe.generic.JsonCodec

object JsonImplicits {
  implicit val config: Configuration =
    Configuration.default.withSnakeCaseMemberNames.withDiscriminator("type")
  implicit val popServiceHealthActionCodec: Codec[PopServiceHealthAction] =
    deriveConfiguredCodec[PopServiceHealthAction]
  implicit val popServicePostgresCodec: Codec[PopServicePostgres] =
    deriveConfiguredCodec[PopServicePostgres]
  implicit val popServiceKafkaCodec: Codec[PopServiceKafka] =
    deriveConfiguredCodec[PopServiceKafka]
  implicit val popServicePopServerCodec: Codec[PopServicePopServer] =
    deriveConfiguredCodec[PopServicePopServer]
  implicit val popServiceKeycloakCodec: Codec[PopServiceKeycloak] =
    deriveConfiguredCodec[PopServiceKeycloak]
  implicit val popServiceCodec: Codec[PopService] =
    deriveConfiguredCodec[PopService]
  implicit val bodyPatternCodec: Codec[BodyPattern] =
    deriveConfiguredCodec[BodyPattern]
  implicit val httpAssertionCodec: Codec[HttpAssertion] =
    deriveConfiguredCodec[HttpAssertion]
  implicit val httpCallCodec: Codec[HttpCall] = deriveConfiguredCodec[HttpCall]
  implicit val kafkaProduceCodec: Codec[KafkaProduce] =
    deriveConfiguredCodec[KafkaProduce]
  implicit val kafkaCheckCodec: Codec[KafkaCheck] =
    deriveConfiguredCodec[KafkaCheck]
  implicit val postgresAssertionCodec: Codec[PostgresAssertion] =
    deriveConfiguredCodec[PostgresAssertion]
  implicit val sqliteAssertionCodec: Codec[SqliteAssertion] =
    deriveConfiguredCodec[SqliteAssertion]
  implicit val sqliteActionCodec: Codec[SqliteAction] =
    deriveConfiguredCodec[SqliteAction]
  implicit val popActionCodec: Codec[PopAction] =
    deriveConfiguredCodec[PopAction]
  implicit val popPostgresCodec: Codec[PopPostgres] =
    deriveConfiguredCodec[PopPostgres]
  implicit val popKafkaCodec: Codec[PopKafka] = deriveConfiguredCodec[PopKafka]
  implicit val popSqliteCodec: Codec[PopSqlite] =
    deriveConfiguredCodec[PopSqlite]
  implicit val popKeycloakCodec: Codec[PopKeycloak] =
    deriveConfiguredCodec[PopKeycloak]
  implicit val popServerBehaviorCodec: Codec[PopServerBehavior] =
    deriveConfiguredCodec[PopServerBehavior]
  implicit val popServerCodec: Codec[PopServer] =
    deriveConfiguredCodec[PopServer]
  implicit val popScenarioCodec: Codec[PopScenario] =
    deriveConfiguredCodec[PopScenario]
}

sealed trait PopServiceHealthAction extends Product with Serializable
case class Http(path: String, patienceSeconds: Int)
    extends PopServiceHealthAction
case class Dummy(alwaysHealthy: Boolean) extends PopServiceHealthAction

case class PopServicePostgres(
    enabled: Option[Boolean] = None,
    hostVar: Option[String] = None,
    portVar: Option[String] = None,
    dbVar: Option[String] = None,
    userVar: Option[String] = None,
    passVar: Option[String] = None
)

case class PopServiceKafka(
    enabled: Option[Boolean] = None,
    bootstrapServerVar: Option[String] = None
)

case class PopServicePopServer(
    enabled: Option[Boolean] = None,
    urlVars: Option[Seq[String]] = None
)

case class PopServiceKeycloak(
    enabled: Option[Boolean] = None,
    urlVar: Option[String] = None,
    realmVar: Option[String] = None
)

case class PopService(
    id: String,
    name: Option[String] = None,
    dir: String,
    baseUrl: String,
    shellPath: Option[String] = None,
    startCmdArgs: Seq[String],
    envVars: Option[Map[String, String]] = None,
    healthCheck: PopServiceHealthAction,
    postgres: Option[PopServicePostgres] = None,
    kafka: Option[PopServiceKafka] = None,
    popServer: Option[PopServicePopServer] = None,
    keycloak: Option[PopServiceKeycloak] = None
)

case class BodyPattern(
    pattern: String,
    condition: String
)

case class HttpAssertion(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    assertCode: Option[Int] = None,
    assertUrl: String,
    serviceId: String,
    method: String,
    jsonPatterns: Option[Seq[BodyPattern]] = None,
    htmlPatterns: Option[Seq[String]] = None,
    withKeycloakUserToken: Option[Boolean] = None,
    messageFilePath: Option[String] = None,
    message: Option[String] = None,
    extraHeaders: Option[Map[String, String]] = None
)

case class HttpCall(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    actionUrl: String,
    serviceId: String,
    withKeycloakUserToken: Option[Boolean] = None,
    method: String,
    messageFilePath: Option[String] = None,
    message: Option[String] = None,
    extraHeaders: Option[Map[String, String]] = None
)

case class KafkaProduce(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    kafkaTopic: String,
    messageFilePath: Option[String] = None,
    message: Option[String] = None
)

case class KafkaCheck(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    check: String
)

case class PostgresAssertion(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    postgresQuery: String,
    assertRowCount: Option[Int] = None,
    assertExpressions: Option[Seq[String]] = None
)

case class SqliteAssertion(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    sqliteQuery: String,
    assertRowCount: Option[Int] = None,
    assertExpressions: Option[Seq[String]] = None
)

case class SqliteAction(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    sqliteAction: String
)

sealed trait PopAction extends Product with Serializable

case class HttpAssertionAction(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    assertCode: Option[Int] = None,
    assertUrl: String,
    serviceId: String,
    method: String,
    jsonPatterns: Option[Seq[BodyPattern]] = None,
    htmlPatterns: Option[Seq[String]] = None,
    withKeycloakUserToken: Option[Boolean] = None,
    messageFilePath: Option[String] = None,
    message: Option[String] = None,
    extraHeaders: Option[Map[String, String]] = None
) extends PopAction

case class HttpCallAction(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    actionUrl: String,
    serviceId: String,
    withKeycloakUserToken: Option[Boolean] = None,
    method: String,
    messageFilePath: Option[String] = None,
    message: Option[String] = None,
    extraHeaders: Option[Map[String, String]] = None
) extends PopAction

case class KafkaProduceAction(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    kafkaTopic: String,
    messageFilePath: Option[String] = None,
    message: Option[String] = None
) extends PopAction

case class KafkaCheckAction(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    check: String
) extends PopAction

case class PostgresAssertionAction(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    postgresQuery: String,
    assertRowCount: Option[Int] = None,
    assertExpressions: Option[Seq[String]] = None
) extends PopAction

case class SqliteAssertionAction(
    sqliteQuery: String,
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    assertRowCount: Option[Int] = None,
    assertExpressions: Option[Seq[String]] = None
) extends PopAction

case class SqliteActionAction(
    name: Option[String] = None,
    delaySeconds: Option[Long] = None,
    sqliteAction: String
) extends PopAction

case class PopPostgres(
    enabled: Option[Boolean] = None,
    imageName: Option[String] = None,
    imageTag: Option[String] = None
)

case class PopKafka(
    enabled: Option[Boolean] = None,
    imageName: Option[String] = None,
    imageTag: Option[String] = None
)

case class PopSqlite(
    dbPath: String
)

case class PopKeycloak(
    enabled: Option[Boolean] = None,
    imageName: Option[String] = None,
    imageTag: Option[String] = None,
    realm: String,
    clientId: String,
    clientSecret: String,
    email: String,
    username: String,
    password: String,
    roles: Option[Seq[String]] = None,
    scopes: Option[Seq[String]] = None
)

case class PopServerBehavior(
    requestPath: String,
    requestMethod: String,
    responseStatus: Int,
    responseContentType: String,
    requestContentType: Option[String] = None,
    responseBodyFile: Option[String] = None,
    responseBody: Option[String] = None
)

case class PopServer(
    enabled: Option[Boolean] = None,
    imageName: Option[String] = None,
    imageTag: Option[String] = None,
    behaviors: Option[Seq[PopServerBehavior]] = None
)

case class PopScenario(
    popServices: Option[Seq[PopService]] = None,
    popActions: Option[Seq[PopAction]] = None,
    popPostgres: Option[PopPostgres] = None,
    popKafka: Option[PopKafka] = None,
    popServer: Option[PopServer] = None,
    popKeycloak: Option[PopKeycloak] = None,
    popSqlite: Option[PopSqlite] = None
)
