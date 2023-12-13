use std::mem::MaybeUninit;

use super::*;

pub struct Inventory<Manager> {
    pub(crate) inventory: *mut sys::ISteamInventory,
    pub(crate) inner: Arc<Inner<Manager>>,
}

impl<Manager> Inventory<Manager> {
    pub fn items_with_prices(&self) -> Vec<SteamItemPrice> {
        unsafe {
            let count = sys::SteamAPI_ISteamInventory_GetNumItemsWithPrices(self.inventory);
            let mut defs = Vec::with_capacity(count as usize);
            let mut prices = Vec::with_capacity(count as usize);
            let mut base_prices = Vec::with_capacity(count as usize);
            sys::SteamAPI_ISteamInventory_GetItemsWithPrices(
                self.inventory,
                defs.as_mut_ptr(),
                prices.as_mut_ptr(),
                base_prices.as_mut_ptr(),
                count,
            );
            defs.into_iter()
                .zip(prices.into_iter())
                .zip(base_prices.into_iter())
                .map(|((item, price), base_price)| SteamItemPrice {
                    item: item,
                    price: price,
                    base_price: base_price,
                })
                .collect()
        }
    }

    pub fn start_purchase<F>(&self, item: SteamItem, quantity: u32, cb: F)
    where
        F: FnOnce(Result<SteamPurchase, SteamError>) + 'static + Send,
    {
        let items = [item.0];
        let quantities = [quantity];
        unsafe {
            let call = sys::SteamAPI_ISteamInventory_StartPurchase(
                self.inventory,
                items.as_ptr(),
                quantities.as_ptr(),
                1,
            );
            register_call_result::<sys::SteamInventoryStartPurchaseResult_t, _, _>(
                &self.inner,
                call,
                5000,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_result != sys::EResult::k_EResultOK {
                        Err(v.m_result.into())
                    } else {
                        Ok(SteamPurchase {
                            order_id: v.m_ulOrderID,
                            transaction_id: v.m_ulTransID,
                        })
                    })
                },
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SteamPurchase {
    pub(crate) order_id: u64,
    pub(crate) transaction_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamItemPrice {
    pub(crate) item: sys::SteamItemDef_t,
    pub(crate) price: u64,
    pub(crate) base_price: u64,
}

impl SteamItemPrice {
    /// The item this price is for
    pub fn item(&self) -> SteamItem {
        SteamItem(self.item)
    }

    /// The price of the item in the local currency
    pub fn price(&self) -> u64 {
        self.price
    }

    /// The base price of the item in the local currency
    pub fn base_price(&self) -> u64 {
        self.base_price
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamItem(pub(crate) sys::SteamItemDef_t);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SteamItemInstance(pub(crate) sys::SteamItemInstanceID_t);
