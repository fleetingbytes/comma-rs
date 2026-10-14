Feature: Upload URL request

  Scenario: Request an upload URL for a known route path
    Given an authenticated client
    And the test dongle ID is available
    When I request an upload URL for the first segment of the most recent route
    Then the response is a JSON array
    And the first element of the response has a string field "url"
