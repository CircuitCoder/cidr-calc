# CIDR Calculator

CIDR addition / subtraction calculator. Maybe useful for setting route tables when metrics are not available (e.g. wireguard next-hop)

```
> let univ = ::/0
> univ - 2001:da8::/56
[
        ::/3
        2000::/16
        2001::/21
        2001:800::/22
        2001:c00::/24
        2001:d00::/25
        2001:d80::/27
        2001:da0::/29
        2001:da8:0:100::/56
        ...
```
