# 1. Upload contracts to the chain (mainnet)
neutrond tx wasm store ../artifacts/vesting.wasm --from ntrn-main-tester --gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y

# Vesting code id = 841
# Vesting factory code id = 842

# 2. Instantiate the vesting factory contract
neutrond tx wasm init 842 '{"vesting_code_id": 841}' --label vesting-factory --no-admin --from ntrn-main-tester --gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y

# Factory address = "neutron1y03rg938hv3t8j0egp5v0cnn2vhnt57exrdcq23gny0933mww5estuar08"

# 3. Create new vesting native
neutrond tx wasm execute neutron1y03rg938hv3t8j0egp5v0cnn2vhnt57exrdcq23gny0933mww5estuar08 '{"create_vesting": {"receiver": {"native": {"address": "neutron1zjjqxfqm33tz27phd0z4jyg53fv0yq7mpxttw0"}}, "vesting_strategy": "hour", "label": "yan-native-vesting"}}' --amount 3600untrn --from ntrn-main-tester --gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y

'{
  "create_vesting": {
    "receiver": {
      "native": {
        "address": "neutron1234"
        }
      },
    "vesting_strategy": "hour",
    "label": "yan-native-vesting"
  }
}'

# Native vesting = "neutron1cpmggumtza3kgkuv292ykt7cef84qc233csd9ddtddcc03z67glsvu5aha"

# 4. Create new vesting IBC
neutrond tx wasm execute neutron1y03rg938hv3t8j0egp5v0cnn2vhnt57exrdcq23gny0933mww5estuar08 '{"create_vesting": {"receiver": {"ibc": {"address": "cosmos1zjjqxfqm33tz27phd0z4jyg53fv0yq7m9ezf5g", "channel_id": "channel-1", "claimer": "neutron1zjjqxfqm33tz27phd0z4jyg53fv0yq7mpxttw0"}}, "vesting_strategy": "hour", "label": "yan-ibc-vesting"}}' --amount 3600untrn --from ntrn-main-tester --gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y

'{
  "create_vesting": {
    "receiver": {
      "ibc": {
        "address": "cosmos1234",
        "channel_id": "channel-1",
        "claimer": "neutron1234"
        }
      },
    "vesting_strategy": "hour",
    "label": "yan-ibc-vesting"
  }
}'

# IBC vesting = neutron17msyv3k4nst33z3ff0x3x0nx8qp77n09tnxhn364ulweduhzhtxqk6qlrs
neutron17msyv3k4nst33z3ff0x3x0nx8qp77n09tnxhn364ulweduhzhtxqk6qlrs
# 5. Query for the vesting addresses
neutrond query wasm contract-state smart neutron1y03rg938hv3t8j0egp5v0cnn2vhnt57exrdcq23gny0933mww5estuar08 '{"get_vesting_addr": {"receiver": "neutron1zjjqxfqm33tz27phd0z4jyg53fv0yq7mpxttw0"}}' --output json
neutrond query wasm contract-state smart neutron1y03rg938hv3t8j0egp5v0cnn2vhnt57exrdcq23gny0933mww5estuar08 '{"get_vesting_addr": {"receiver": "cosmos1zjjqxfqm33tz27phd0z4jyg53fv0yq7m9ezf5g"}}' --output json

# 6. Query for claimable amount
neutrond query wasm contract-state smart neutron1cpmggumtza3kgkuv292ykt7cef84qc233csd9ddtddcc03z67glsvu5aha '{"get_claimable": {}}' --output json
neutrond query wasm contract-state smart neutron17msyv3k4nst33z3ff0x3x0nx8qp77n09tnxhn364ulweduhzhtxqk6qlrs '{"get_claimable": {}}' --output json

# 7. Claim the amount
neutrond tx wasm execute neutron1cpmggumtza3kgkuv292ykt7cef84qc233csd9ddtddcc03z67glsvu5aha '{"claim": {}}' --from main --gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y
neutrond tx wasm execute neutron17msyv3k4nst33z3ff0x3x0nx8qp77n09tnxhn364ulweduhzhtxqk6qlrs '{"claim": {}}' --from main --gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y

