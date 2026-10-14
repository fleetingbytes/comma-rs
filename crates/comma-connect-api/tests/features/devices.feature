Feature: Device listing

  Scenario: List devices accessible to the authenticated user
    Given an authenticated client
    When I request the list of devices
    Then the response is a JSON array
    And every element of the response is an object with a string field "dongle_id"
