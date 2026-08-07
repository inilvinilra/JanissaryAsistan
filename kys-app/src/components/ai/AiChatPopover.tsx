import { BotIcon, ChevronDownIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import { ButtonGroup } from "@/components/ui/button-group"
import {
  Field,
  FieldDescription,
  FieldLabel,
} from "@/components/ui/field"
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "@/components/ui/popover"
import { Textarea } from "@/components/ui/textarea"

export function AiChatPopover() {
  return (
    <ButtonGroup>
      <Button variant="outline">
        <BotIcon className="mr-2" /> KYS Asistan
      </Button>
      <Popover>
        <PopoverTrigger render={<Button variant="outline" size="icon" aria-label="Open Popover"><ChevronDownIcon /></Button>} />
        <PopoverContent align="end" className="rounded-xl text-sm w-80">
          <PopoverHeader>
            <PopoverTitle>Yapay Zeka ile İncele</PopoverTitle>
            <PopoverDescription>
              Proje hakkında yapay zekaya dilediğinizi sorabilirsiniz.
            </PopoverDescription>
          </PopoverHeader>
          <Field>
            <FieldLabel htmlFor="task" className="sr-only">
              Görev Açıklaması
            </FieldLabel>
            <Textarea
              id="task"
              placeholder="Örn: Bu projenin en büyük eksikliği nedir?"
              className="resize-none mt-2"
            />
            <FieldDescription className="mt-2">
              KYS Asistan, analiz raporunu sizin için yorumlayacaktır.
            </FieldDescription>
          </Field>
        </PopoverContent>
      </Popover>
    </ButtonGroup>
  )
}

export default AiChatPopover;
