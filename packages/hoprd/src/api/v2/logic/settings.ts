import Hopr, { PassiveStrategy, PromiscuousStrategy } from "@hoprnet/hopr-core"
import { APIv2State } from "../../v2"

export interface Setting {
    name: string
    value: any
}

export interface APIv2Settings {
    includeRecipient: boolean,
    strategy: "passive" | "promiscuous"
}

export const getSetting = ({ node, state, settingName }: { node: Hopr, state: APIv2State, settingName?: keyof APIv2Settings }) => {
    const getSettingByName = (name: string): Setting | Error => {
        if (name) {
            const setting = state.settings[name]
            if (setting === undefined) {
                return new Error("invalidSettingName")
            }

            switch (name) {
                case "strategy":
                    return { name, value: node.getChannelStrategy() }
            }
            return { name, value: setting }
        }
    }

    if (!settingName) {
        return Array.from(Object.keys(state.settings)).map(name => ({ name, value: getSettingByName(name) }))
    }
    return getSettingByName(settingName)

}

export const setSetting = ({ node, settingName, state, value }: { settingName: keyof APIv2Settings, value: any, node: Hopr, state: APIv2State }) => {
    if (state.settings[settingName] === undefined) {
        return new Error("invalidSettingName")
    }

    switch (settingName) {
        case "includeRecipient":
            if (typeof value !== "boolean") return new Error("invalidValue")
            state.settings[settingName] = value
            break
        case "strategy":
            let strategy

            switch (value) {
                case "passive":
                    strategy = new PassiveStrategy()
                    break;
                case "promiscuous":
                    strategy = new PromiscuousStrategy()
                    break;
            }
            if (!strategy) return new Error("invalidValue")
            node.setChannelStrategy(strategy)
            state.settings[settingName] = value
            break
    }
}
