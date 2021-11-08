import { RegistryTypes } from "@moonrabbit/types/types";
import fs from "fs";
import {  mrevmDefinitions } from ".";

async function generateJSON() {
  const version = process.argv[2] || "latest";
  let types: RegistryTypes;
  if (! mrevmDefinitions.types) {
    throw new Error("missing types definitions");
  } else if (version === "latest") {
    types =  mrevmDefinitions.types[ mrevmDefinitions.types.length - 1].types;
  } else if (Number(version)) {
    let i = 0;
    while (
      i <  mrevmDefinitions.types.length &&
       mrevmDefinitions.types[i].minmax[1] &&
      Number( mrevmDefinitions.types[i].minmax[1]) < Number(version)
    ) {
      i += 1;
    }
    types =  mrevmDefinitions.types[i].types;
  } else {
    throw new Error("parameter must be number or `latest`");
  }
  console.log(JSON.stringify(types));
  fs.appendFile(" mrevm-types-" + version + ".json", JSON.stringify(types), function (err) {
    if (err) throw err;
    console.log("Saved for version : " + version);
  });
}
generateJSON();
