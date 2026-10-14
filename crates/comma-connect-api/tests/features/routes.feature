Feature: Routes and route files

  Scenario: List recent routes for the configured dongle
    Given an authenticated client
    And the test dongle ID is available
    When I request the most recent routes for the dongle
    Then the response is a JSON array
    And if the array is non-empty, every element has a string field "fullname"

  Scenario: Fetch uploaded files for a recent route
    Given an authenticated client
    And the test dongle ID is available
    When I request the uploaded files for the most recent route of the dongle
    Then the response is a JSON object
    And the response contains an array field "qlogs"
    And the response contains an array field "logs"
