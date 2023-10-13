use alloy_sol_macro::sol;

use alloy_sol_types::{eip712_domain, Eip712Domain};

use once_cell::sync::Lazy;

use crate::constants::{CURRENT_SEAPORT_ADDRESS, CURRENT_SEAPORT_VERSION};

pub static SEAPORT_DOMAIN: Lazy<Eip712Domain> = Lazy::new(|| {
    eip712_domain! {
        name: String::from("Seaport"),
        version: String::from(CURRENT_SEAPORT_VERSION),
        chain_id: 5,
        verifying_contract: CURRENT_SEAPORT_ADDRESS,
    }
});

sol! {
    enum OrderType {
        FULL_OPEN,
        PARTIAL_OPEN,
        FULL_RESTRICTED,
        PARTIAL_RESTRICTED,
        CONTRACT
    }

    enum ItemType {
        NATIVE,
        ERC20,
        ERC721,
        ERC1155,
        ERC721_WITH_CRITERIA,
        ERC1155_WITH_CRITERIA
    }

    struct OfferItem {
        ItemType itemType;
        address token;
        uint256 identifierOrCriteria;
        uint256 startAmount;
        uint256 endAmount;
    }

    struct OrderComponents {
        address offerer;
        address zone;
        OfferItem[] offer;
        ConsiderationItem[] consideration;
        OrderType orderType;
        uint256 startTime;
        uint256 endTime;
        bytes32 zoneHash;
        uint256 salt;
        bytes32 conduitKey;
        uint256 counter;
    }

    struct ConsiderationItem {
        ItemType itemType;
        address token;
        uint256 identifierOrCriteria;
        uint256 startAmount;
        uint256 endAmount;
        address payable recipient;
    }

    struct ReceivedItem {
        ItemType itemType;
        address token;
        uint256 identifier;
        uint256 amount;
        address payable recipient;
    }

    struct BasicOrderParameters {
        address considerationToken;
        uint256 considerationIdentifier;
        uint256 considerationAmount;
        address payable offerer;
        address zone;
        address offerToken;
        uint256 offerIdentifier;
        uint256 offerAmount;
        BasicOrderType basicOrderType;
        uint256 startTime;
        uint256 endTime;
        bytes32 zoneHash;
        uint256 salt;
        bytes32 offererConduitKey;
        bytes32 fulfillerConduitKey;
        uint256 totalOriginalAdditionalRecipients;
        AdditionalRecipient[] additionalRecipients;
        bytes signature;
    }

    struct OrderParameters {
        address offerer;
        address zone;
        OfferItem[] offer;
        ConsiderationItem[] consideration;
        OrderType orderType;
        uint256 startTime;
        uint256 endTime;
        bytes32 zoneHash;
        uint256 salt;
        bytes32 conduitKey;
        uint256 totalOriginalConsiderationItems;
    }

    struct Order {
        OrderParameters parameters;
        bytes signature;
    }

    struct AdvancedOrder {
        OrderParameters parameters;
        uint120 numerator;
        uint120 denominator;
        bytes signature;
        bytes extraData;
    }

    struct OrderStatus {
        bool isValidated;
        bool isCancelled;
        uint120 numerator;
        uint120 denominator;
    }

    struct ZoneParameters {
        bytes32 orderHash;
        address fulfiller;
        address offerer;
        SpentItem[] offer;
        ReceivedItem[] consideration;
        bytes extraData;
        bytes32[] orderHashes;
        uint256 startTime;
        uint256 endTime;
        bytes32 zoneHash;
    }

    struct SpentItem {
        ItemType itemType;
        address token;
        uint256 identifier;
        uint256 amount;
    }

    struct AdditionalRecipient {
        uint256 amount;
        address payable recipient;
    }

    enum BasicOrderType {
        ETH_TO_ERC721_FULL_OPEN,
        ETH_TO_ERC721_PARTIAL_OPEN,
        ETH_TO_ERC721_FULL_RESTRICTED,
        ETH_TO_ERC721_PARTIAL_RESTRICTED,
        ETH_TO_ERC1155_FULL_OPEN,
        ETH_TO_ERC1155_PARTIAL_OPEN,
        ETH_TO_ERC1155_FULL_RESTRICTED,
        ETH_TO_ERC1155_PARTIAL_RESTRICTED,
        ERC20_TO_ERC721_FULL_OPEN,
        ERC20_TO_ERC721_PARTIAL_OPEN,
        ERC20_TO_ERC721_FULL_RESTRICTED,
        ERC20_TO_ERC721_PARTIAL_RESTRICTED,
        ERC20_TO_ERC1155_FULL_OPEN,
        ERC20_TO_ERC1155_PARTIAL_OPEN,
        ERC20_TO_ERC1155_FULL_RESTRICTED,
        ERC20_TO_ERC1155_PARTIAL_RESTRICTED,
        ERC721_TO_ERC20_FULL_OPEN,
        ERC721_TO_ERC20_PARTIAL_OPEN,
        ERC721_TO_ERC20_FULL_RESTRICTED,
        ERC721_TO_ERC20_PARTIAL_RESTRICTED,
        ERC1155_TO_ERC20_FULL_OPEN,
        ERC1155_TO_ERC20_PARTIAL_OPEN,
        ERC1155_TO_ERC20_FULL_RESTRICTED,
        ERC1155_TO_ERC20_PARTIAL_RESTRICTED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_lazy() {
        let dom = &*SEAPORT_DOMAIN;
        println!("{:?}", dom);
    }
}