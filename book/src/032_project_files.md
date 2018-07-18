# Project Files

## folder structure


```bash
# root dir
├── working
│   └── Project1
│       └── Project1.yml
├── archive
│   ├── 2018
│   └── 2019
│       └── R036_Project3
│           ├── Project3.yml
│           └── R036 Project3 2019-10-08.tex
...
```

## asciii yaml files

### client section


```yaml
--- # asciii document  (version: 3.1.1 - commit 452 (2636464) (2017-09-20, release), template: default)
# vim:set ft=yaml:

client:
  title: Frau    # Herr # Frau # Mr, Ms, Mrs
  first_name: Hendrik
  last_name: Sollich

  email:
  address: |
    Hendrik Sollich
    Somewhere in Dresden
    Elmstreet 13
    
    01062 Dresden

```


### event section

```yaml
event:
  name: asciii release party
  location: # might be a list
  dates:
  - begin: 10.01.2019
    #end:
    times:
    -  begin: "13.00"
       end:   "18.00"

  description: |
    ##DESCRIPTION##
```

### offer section

```yaml
offer:
  date: 27.11.2017
  appendix: 1
```


### invoice section

### hours section

### products section

```yaml
  "Sekt  (0,75l)":
    amount: 4
    price: 6.0
    sold: 2
  "Belegte Brötchen":
    amount: 90
    price: 1.16
```

```yaml
cataloge:
  product: &kaffee       { name: Kaffee, price: 2.5, unit: 1l  }

products:
  *kaffee:
    amount: 60
```

