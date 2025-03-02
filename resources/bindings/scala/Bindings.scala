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
  implicit val config: Configuration = Configuration.default.withSnakeCaseMemberNames.withDiscriminator("type")
  implicit val popServiceHealthActionCodec: Codec[PopServiceHealthAction] = deriveConfiguredCodec[PopServiceHealthAction]
  implicit val popServicePostgresCodec: Codec[PopServicePostgres] = deriveConfiguredCodec[PopServicePostgres]
  implicit val popServiceKafkaCodec: Codec[PopServiceKafka] = deriveConfiguredCodec[PopServiceKafka]
  implicit val popServicePopServerCodec: Codec[PopServicePopServer] = deriveConfiguredCodec[PopServicePopServer]
  implicit val popServiceKeycloakCodec: Codec[PopServiceKeycloak] = deriveConfiguredCodec[PopServiceKeycloak]
  implicit val popServiceCodec: Codec[PopService] = deriveConfiguredCodec[PopService]
  implicit val bodyPatternCodec: Codec[BodyPattern] = deriveConfiguredCodec[BodyPattern]
  implicit val httpAssertionCodec: Codec[HttpAssertion] = deriveConfiguredCodec[HttpAssertion]
  implicit val httpCallCodec: Codec[HttpCall] = deriveConfiguredCodec[HttpCall]
  implicit val kafkaProduceCodec: Codec[KafkaProduce] = deriveConfiguredCodec[KafkaProduce]
  implicit val kafkaCheckCodec: Codec[KafkaCheck] = deriveConfiguredCodec[KafkaCheck]
  implicit val postgresAssertionCodec: Codec[PostgresAssertion] = deriveConfiguredCodec[PostgresAssertion]
  implicit val sqliteAssertionCodec: Codec[SqliteAssertion] = deriveConfiguredCodec[SqliteAssertion]
  implicit val sqliteActionCodec: Codec[SqliteAction] = deriveConfiguredCodec[SqliteAction]
  implicit val popActionCodec: Codec[PopAction] = deriveConfiguredCodec[PopAction]
  implicit val popPostgresCodec: Codec[PopPostgres] = deriveConfiguredCodec[PopPostgres]
  implicit val popKafkaCodec: Codec[PopKafka] = deriveConfiguredCodec[PopKafka]
  implicit val popSqliteCodec: Codec[PopSqlite] = deriveConfiguredCodec[PopSqlite]
  implicit val popKeycloakCodec: Codec[PopKeycloak] = deriveConfiguredCodec[PopKeycloak]
  implicit val popServerBehaviorCodec: Codec[PopServerBehavior] = deriveConfiguredCodec[PopServerBehavior]
  implicit val popServerCodec: Codec[PopServer] = deriveConfiguredCodec[PopServer]
  implicit val popScenarioCodec: Codec[PopScenario] = deriveConfiguredCodec[PopScenario]
}


sealed trait PopServiceHealthAction extends Product with Serializable
case class Http(path: String, patienceSeconds: Int) extends PopServiceHealthAction
case class Dummy(alwaysHealthy: Boolean) extends PopServiceHealthAction

case class PopServicePostgres(
  enabled: Option[Boolean],
  hostVar: Option[String],
  portVar: Option[String],
  dbVar: Option[String],
  userVar: Option[String],
  passVar: Option[String]
)

case class PopServiceKafka(
  enabled: Option[Boolean],
  bootstrapServerVar: Option[String]
)

case class PopServicePopServer(
  enabled: Option[Boolean],
  urlVars: Option[Seq[String]]
)

case class PopServiceKeycloak(
  enabled: Option[Boolean],
  urlVar: Option[String],
  realmVar: Option[String]
)

case class PopService(
  id: String,
  name: Option[String],
  dir: String,
  baseUrl: String,
  shellPath: Option[String],
  startCmdArgs: Seq[String],
  envVars: Option[Map[String, String]],
  healthCheck: PopServiceHealthAction,
  postgres: Option[PopServicePostgres],
  kafka: Option[PopServiceKafka],
  popServer: Option[PopServicePopServer],
  keycloak: Option[PopServiceKeycloak]
)

case class BodyPattern(
  pattern: String,
  condition: String
)

case class HttpAssertion(
  name: Option[String],
  delaySeconds: Option[Long],
  assertCode: Option[Short],
  assertUrl: String,
  serviceId: String,
  method: String,
  jsonPatterns: Option[Seq[BodyPattern]],
  htmlPatterns: Option[Seq[String]],
  withKeycloakUserToken: Option[Boolean],
  messageFilePath: Option[String],
  message: Option[String],
  extraHeaders: Option[Map[String, String]]
)

case class HttpCall(
  name: Option[String],
  delaySeconds: Option[Long],
  actionUrl: String,
  serviceId: String,
  withKeycloakUserToken: Option[Boolean],
  method: String,
  messageFilePath: Option[String],
  message: Option[String],
  extraHeaders: Option[Map[String, String]]
)

case class KafkaProduce(
  name: Option[String],
  delaySeconds: Option[Long],
  kafkaTopic: String,
  messageFilePath: Option[String],
  message: Option[String]
)

case class KafkaCheck(
  name: Option[String],
  delaySeconds: Option[Long],
  check: String
)

case class PostgresAssertion(
  name: Option[String],
  delaySeconds: Option[Long],
  postgresQuery: String,
  assertRowCount: Option[Int],
  assertExpressions: Option[Seq[String]]
)

case class SqliteAssertion(
  name: Option[String],
  delaySeconds: Option[Long],
  sqliteQuery: String,
  assertRowCount: Option[Int],
  assertExpressions: Option[Seq[String]]
)

case class SqliteAction(
  name: Option[String],
  delaySeconds: Option[Long],
  sqliteAction: String
)

sealed trait PopAction extends Product with Serializable

case class HttpAssertionAction(
  name: Option[String],
  delaySeconds: Option[Long],
  assertCode: Option[Short],
  assertUrl: String,
  serviceId: String,
  method: String,
  jsonPatterns: Option[Seq[BodyPattern]],
  htmlPatterns: Option[Seq[String]],
  withKeycloakUserToken: Option[Boolean],
  messageFilePath: Option[String],
  message: Option[String],
  extraHeaders: Option[Map[String, String]]
) extends PopAction

case class HttpCallAction(
  name: Option[String],
  delaySeconds: Option[Long],
  actionUrl: String,
  serviceId: String,
  withKeycloakUserToken: Option[Boolean],
  method: String,
  messageFilePath: Option[String],
  message: Option[String],
  extraHeaders: Option[Map[String, String]]
) extends PopAction

case class KafkaProduceAction(
  name: Option[String],
  delaySeconds: Option[Long],
  kafkaTopic: String,
  messageFilePath: Option[String],
  message: Option[String]
) extends PopAction

case class KafkaCheckAction(
  name: Option[String],
  delaySeconds: Option[Long],
  check: String
) extends PopAction

case class PostgresAssertionAction(
  name: Option[String],
  delaySeconds: Option[Long],
  postgresQuery: String,
  assertRowCount: Option[Int],
  assertExpressions: Option[Seq[String]]
) extends PopAction

case class SqliteAssertionAction(
  name: Option[String],
  delaySeconds: Option[Long],
  sqliteQuery: String,
  assertRowCount: Option[Int],
  assertExpressions: Option[Seq[String]]
) extends PopAction

case class SqliteActionAction(
  name: Option[String],
  delaySeconds: Option[Long],
  sqliteAction: String
) extends PopAction

case class PopPostgres(
  enabled: Option[Boolean],
  imageName: Option[String],
  imageTag: Option[String]
)

case class PopKafka(
  enabled: Option[Boolean],
  imageName: Option[String],
  imageTag: Option[String]
)

case class PopSqlite(
  dbPath: String
)

case class PopKeycloak(
  enabled: Option[Boolean],
  imageName: Option[String],
  imageTag: Option[String],
  realm: String,
  clientId: String,
  clientSecret: String,
  email: String,
  username: String,
  password: String,
  roles: Option[Seq[String]],
  scopes: Option[Seq[String]]
)

case class PopServerBehavior(
  requestPath: String,
  requestMethod: String,
  responseStatus: Short,
  requestContentType: Option[String],
  responseContentType: String,
  responseBodyFile: Option[String],
  responseBody: Option[String]
)

case class PopServer(
  enabled: Option[Boolean],
  imageName: Option[String],
  imageTag: Option[String],
  behaviors: Option[Seq[PopServerBehavior]]
)

case class PopScenario(
  popServices: Option[Seq[PopService]],
  popActions: Option[Seq[PopAction]],
  popPostgres: Option[PopPostgres],
  popKafka: Option[PopKafka],
  popServer: Option[PopServer],
  popKeycloak: Option[PopKeycloak],
  popSqlite: Option[PopSqlite]
)


