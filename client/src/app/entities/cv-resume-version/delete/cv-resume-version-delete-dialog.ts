import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap/modal';

import { ITEM_DELETED_EVENT } from 'app/config/navigation.constants';
import { AlertError } from 'app/shared/alert/alert-error';
import { TranslateDirective } from 'app/shared/language';
import { ICvResumeVersion } from '../cv-resume-version.model';
import { CvResumeVersionService } from '../service/cv-resume-version.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './cv-resume-version-delete-dialog.html',
  imports: [TranslateDirective, FormsModule, FontAwesomeModule, AlertError],
})
export class CvResumeVersionDeleteDialog {
  cvResumeVersion?: ICvResumeVersion;

  protected readonly cvResumeVersionService = inject(CvResumeVersionService);
  protected readonly activeModal = inject(NgbActiveModal);

  cancel(): void {
    this.activeModal.dismiss();
  }

  confirmDelete(id: number): void {
    this.cvResumeVersionService.delete(id).subscribe(() => {
      this.activeModal.close(ITEM_DELETED_EVENT);
    });
  }
}
