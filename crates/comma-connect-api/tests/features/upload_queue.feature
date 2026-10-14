Feature: Athena upload queue

  Scenario: Query the upload queue for the configured dongle
    Given an authenticated client
    And the test dongle ID is available
    When I request the upload queue for the dongle
    Then the response is a JSON array
