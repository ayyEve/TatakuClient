# get the client version
$CLIENT_VERSION = Get-Content -Path "./.client"

# setup the url
$Url = "https://releases.tataku.ca/${CI_BRANCH_NAME}/${CLIENT_VERSION}"

# build form body
$Form = @{
    token = $TATAKU_RELEASE_KEY
    filename = "tataku-client.exe"
    file = Get-Item -Path "./target/release/tataku-client.exe"
}

# send it off
Invoke-RestMethod -Method "PUT" -Uri $Url -Form $Form