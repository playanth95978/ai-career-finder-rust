import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { ComponentFixture, TestBed } from '@angular/core/testing';

import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap/modal';
import { of } from 'rxjs';

import { JobApplicationService } from '../service/job-application.service';

import { JobApplicationDeleteDialog } from './job-application-delete-dialog';

describe('JobApplication Management Delete Component', () => {
  let comp: JobApplicationDeleteDialog;
  let fixture: ComponentFixture<JobApplicationDeleteDialog>;
  let service: JobApplicationService;
  let mockActiveModal: NgbActiveModal;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [NgbActiveModal],
    });
    fixture = TestBed.createComponent(JobApplicationDeleteDialog);
    comp = fixture.componentInstance;
    service = TestBed.inject(JobApplicationService);
    mockActiveModal = TestBed.inject(NgbActiveModal);
  });

  describe('confirmDelete', () => {
    it('should call delete service on confirmDelete', () => {
      // GIVEN
      vitest.spyOn(service, 'delete').mockReturnValue(of(undefined));
      vitest.spyOn(mockActiveModal, 'close');

      // WHEN
      comp.confirmDelete(123);

      // THEN
      expect(service.delete).toHaveBeenCalledWith(123);
      expect(mockActiveModal.close).toHaveBeenCalledWith('deleted');
    });

    it('should not call delete service on clear', () => {
      // GIVEN
      vitest.spyOn(service, 'delete');
      vitest.spyOn(mockActiveModal, 'close');
      vitest.spyOn(mockActiveModal, 'dismiss');

      // WHEN
      comp.cancel();

      // THEN
      expect(service.delete).not.toHaveBeenCalled();
      expect(mockActiveModal.close).not.toHaveBeenCalled();
      expect(mockActiveModal.dismiss).toHaveBeenCalled();
    });
  });
});
