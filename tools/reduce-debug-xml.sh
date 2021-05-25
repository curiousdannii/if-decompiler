#!/bin/bash

# Remove information from the gameinfo.dbg that we don't need

pattern=""
pattern+="<action>.+?</action>|"
pattern+="<array>.+?</array>|"
pattern+="<attribute>.+?</attribute>|"
pattern+="<class>.+?</class>|"
pattern+="<constant>.+?</constant>|"
pattern+="<global-variable>.+?</global-variable>|"
pattern+="<local-variable>.+?</local-variable>|"
pattern+="<object>.+?</object>|"
pattern+="<property>.+?</property>|"
pattern+="<sequence-point>.+?</sequence-point>|"
pattern+="<source-code-location>.+?</source-code-location>|"
pattern+="<story-file-section>.+?</story-file-section>|"
pattern+="<table-entry>.+?</table-entry>"

cat "$1" | perl -pe "s#($pattern)##g" > "$1.reduced"