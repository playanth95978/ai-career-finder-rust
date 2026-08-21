import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap/modal';

import { ITEM_DELETED_EVENT } from 'app/config/navigation.constants';
import { AlertError } from 'app/shared/alert/alert-error';
import { TranslateDirective } from 'app/shared/language';
import { IAutoApplyConfig } from '../auto-apply-config.model';
import { AutoApplyConfigService } from '../service/auto-apply-config.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './auto-apply-config-delete-dialog.html',
  imports: [TranslateDirective, FormsModule, FontAwesomeModule, AlertError],
})
export class AutoApplyConfigDeleteDialog {
  autoApplyConfig?: IAutoApplyConfig;

  protected readonly autoApplyConfigService = inject(AutoApplyConfigService);
  protected readonly activeModal = inject(NgbActiveModal);

  cancel(): void {
    this.activeModal.dismiss();
  }

  confirmDelete(id: number): void {
    this.autoApplyConfigService.delete(id).subscribe(() => {
      this.activeModal.close(ITEM_DELETED_EVENT);
    });
  }
}
