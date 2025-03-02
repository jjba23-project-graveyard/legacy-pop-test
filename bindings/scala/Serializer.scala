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

package poptest

import java.io.PrintWriter
import java.nio.file.Paths
import io.circe.Codec
import io.circe.generic.extras.Configuration
import io.circe.generic.extras.semiauto._
import io.circe.generic.JsonCodec
import poptest.models._
import io.circe.syntax._
import poptest.models.JsonImplicits._

object Serializer {
  def serializeToFile(args: Array[String], scenario: PopScenario) = {
    if (args.length != 1) {
      println("A file path must be provided to write to!")
      System.exit(1)
    }

    val filePath = args(0)

    var writer: PrintWriter = null

    try {
      writer = new PrintWriter(Paths.get(filePath).toFile)
      val json: String = scenario.asJson.deepDropNullValues.spaces2
      writer.write(json)
      println(s"Successfully wrote to file: $filePath")
    } catch {
      case e: Exception =>
        println(s"An error occurred while writing to file: ${e.getMessage}")
    } finally {
      if (writer != null) {
        writer.close()
      }
    }
  }
}
